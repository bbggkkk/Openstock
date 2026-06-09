use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize)]
pub struct CacheStatus {
    pub root: String,
    pub exists: bool,
    pub total_files: usize,
    pub total_bytes: u64,
    pub namespaces: Vec<CacheNamespaceStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheNamespaceStatus {
    pub namespace: String,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CachePrunePolicy {
    pub namespace: String,
    pub directory: String,
    pub max_snapshot_files: usize,
    pub max_snapshot_bytes: u64,
    pub protected_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CachePruneReport {
    pub policy: CachePrunePolicy,
    pub dry_run: bool,
    pub deleted_files: usize,
    pub deleted_bytes: u64,
    pub retained_snapshot_files: usize,
    pub retained_snapshot_bytes: u64,
    pub deleted_paths: Vec<String>,
}

#[derive(Debug, Clone)]
struct SnapshotFile {
    path: PathBuf,
    name: String,
    bytes: u64,
    modified: SystemTime,
}

pub fn status() -> Result<CacheStatus, String> {
    let root = crate::core::paths::cache_dir();
    let root = root.as_path();
    if !root.exists() {
        return Ok(CacheStatus {
            root: root.display().to_string(),
            exists: false,
            total_files: 0,
            total_bytes: 0,
            namespaces: Vec::new(),
        });
    }

    let mut namespaces = BTreeMap::<String, CacheNamespaceStatus>::new();
    let mut total_files = 0;
    let mut total_bytes = 0;
    collect_status(
        root,
        root,
        &mut namespaces,
        &mut total_files,
        &mut total_bytes,
    )?;

    Ok(CacheStatus {
        root: root.display().to_string(),
        exists: true,
        total_files,
        total_bytes,
        namespaces: namespaces.into_values().collect(),
    })
}

pub fn prune_json_snapshots(
    namespace: &str,
    dir: &Path,
    protected_files: &[&str],
    max_snapshot_files: usize,
    max_snapshot_bytes: u64,
    dry_run: bool,
) -> Result<CachePruneReport, String> {
    prune_snapshots(
        namespace,
        dir,
        protected_files,
        &["json"],
        max_snapshot_files,
        max_snapshot_bytes,
        dry_run,
    )
}

pub fn prune_snapshots(
    namespace: &str,
    dir: &Path,
    protected_files: &[&str],
    extensions: &[&str],
    max_snapshot_files: usize,
    max_snapshot_bytes: u64,
    dry_run: bool,
) -> Result<CachePruneReport, String> {
    let policy = CachePrunePolicy {
        namespace: namespace.to_string(),
        directory: dir.display().to_string(),
        max_snapshot_files,
        max_snapshot_bytes,
        protected_files: protected_files
            .iter()
            .map(|value| value.to_string())
            .collect(),
    };

    if !dir.exists() {
        return Ok(CachePruneReport {
            policy,
            dry_run,
            deleted_files: 0,
            deleted_bytes: 0,
            retained_snapshot_files: 0,
            retained_snapshot_bytes: 0,
            deleted_paths: Vec::new(),
        });
    }

    let mut snapshots = snapshot_files(dir, protected_files, extensions)?;
    snapshots.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| b.name.cmp(&a.name))
    });

    let mut retained_bytes = 0_u64;
    let mut delete = Vec::new();
    for (index, snapshot) in snapshots.iter().enumerate() {
        let keep_by_count = index < max_snapshot_files;
        let keep_by_bytes = retained_bytes.saturating_add(snapshot.bytes) <= max_snapshot_bytes;
        if keep_by_count && keep_by_bytes {
            retained_bytes = retained_bytes.saturating_add(snapshot.bytes);
        } else {
            delete.push(snapshot.clone());
        }
    }

    let deleted_files = delete.len();
    let deleted_bytes = delete.iter().map(|file| file.bytes).sum::<u64>();
    let deleted_paths = delete
        .iter()
        .map(|file| file.path.display().to_string())
        .collect::<Vec<_>>();

    if !dry_run {
        for file in &delete {
            fs::remove_file(&file.path).map_err(|err| {
                format!(
                    "[cache] 캐시 파일 삭제 실패 ({}): {}",
                    file.path.display(),
                    err
                )
            })?;
        }
    }

    Ok(CachePruneReport {
        policy,
        dry_run,
        deleted_files,
        deleted_bytes,
        retained_snapshot_files: snapshots.len().saturating_sub(deleted_files),
        retained_snapshot_bytes: retained_bytes,
        deleted_paths,
    })
}

fn collect_status(
    root: &Path,
    dir: &Path,
    namespaces: &mut BTreeMap<String, CacheNamespaceStatus>,
    total_files: &mut usize,
    total_bytes: &mut u64,
) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|err| format!("[cache] 캐시 디렉터리 읽기 실패: {}", err))?
    {
        let entry = entry.map_err(|err| format!("[cache] 캐시 항목 읽기 실패: {}", err))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|err| format!("[cache] 캐시 메타데이터 읽기 실패: {}", err))?;
        if metadata.is_dir() {
            collect_status(root, &path, namespaces, total_files, total_bytes)?;
        } else if metadata.is_file() {
            let bytes = metadata.len();
            *total_files += 1;
            *total_bytes = total_bytes.saturating_add(bytes);
            let namespace = namespace_for(root, &path);
            let entry = namespaces
                .entry(namespace.clone())
                .or_insert(CacheNamespaceStatus {
                    namespace,
                    files: 0,
                    bytes: 0,
                });
            entry.files += 1;
            entry.bytes = entry.bytes.saturating_add(bytes);
        }
    }
    Ok(())
}

fn namespace_for(root: &Path, path: &Path) -> String {
    let Ok(relative) = path.strip_prefix(root) else {
        return "unknown".to_string();
    };
    relative
        .components()
        .next()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| "root".to_string())
}

fn snapshot_files(
    dir: &Path,
    protected_files: &[&str],
    extensions: &[&str],
) -> Result<Vec<SnapshotFile>, String> {
    let mut files = Vec::new();
    for entry in
        fs::read_dir(dir).map_err(|err| format!("[cache] 캐시 디렉터리 읽기 실패: {}", err))?
    {
        let entry = entry.map_err(|err| format!("[cache] 캐시 항목 읽기 실패: {}", err))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|err| format!("[cache] 캐시 메타데이터 읽기 실패: {}", err))?;
        if !metadata.is_file() {
            continue;
        }
        let Some(name) = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if protected_files.iter().any(|protected| protected == &name)
            || !extensions.iter().any(|extension| {
                path.extension()
                    .and_then(|value| value.to_str())
                    .map(|value| value.eq_ignore_ascii_case(extension))
                    .unwrap_or(false)
            })
        {
            continue;
        }
        files.push(SnapshotFile {
            path,
            name,
            bytes: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        });
    }
    Ok(files)
}
