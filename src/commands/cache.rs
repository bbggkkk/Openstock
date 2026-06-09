use crate::providers::{kind, opendart};
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum CacheCommands {
    /// 로컬 캐시 용량 상태 조회
    Status,

    /// 보존 정책에 따라 오래된 캐시 스냅샷 정리
    Prune(CachePruneCommand),
}

#[derive(Args)]
pub struct CachePruneCommand {
    /// 실제 삭제하지 않고 삭제 예정 파일만 계산
    #[arg(long)]
    pub dry_run: bool,
}

pub fn handle_cache(sub: &CacheCommands) {
    match sub {
        CacheCommands::Status => match crate::core::cache::status() {
            Ok(status) => println!(
                "{}",
                crate::core::output::explained_with_raw(
                    "cache status",
                    "로컬 캐시 용량 상태 조회 결과",
                    vec![
                        crate::core::output::field(
                            "root",
                            "캐시 루트 디렉터리",
                            serde_json::json!(status.root),
                        ),
                        crate::core::output::field(
                            "exists",
                            "캐시 루트 디렉터리 존재 여부",
                            serde_json::json!(status.exists),
                        ),
                        crate::core::output::field(
                            "total_files",
                            "캐시 파일 총 개수",
                            serde_json::json!(status.total_files),
                        ),
                        crate::core::output::field(
                            "total_bytes",
                            "캐시 총 용량(bytes)",
                            serde_json::json!(status.total_bytes),
                        ),
                        crate::core::output::field(
                            "namespaces",
                            "최상위 캐시 namespace별 파일 수와 용량",
                            serde_json::json!(status.namespaces),
                        ),
                    ],
                    serde_json::json!(status),
                )
            ),
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("cache status", "캐시 상태 조회 실패", &err)
            ),
        },
        CacheCommands::Prune(command) => {
            let reports = match prune_all(command.dry_run) {
                Ok(reports) => reports,
                Err(err) => {
                    eprintln!(
                        "{}",
                        crate::core::output::error("cache prune", "캐시 정리 실패", &err)
                    );
                    return;
                }
            };
            let deleted_files = reports
                .iter()
                .map(|report| report.deleted_files)
                .sum::<usize>();
            let deleted_bytes = reports
                .iter()
                .map(|report| report.deleted_bytes)
                .sum::<u64>();
            println!(
                "{}",
                crate::core::output::explained_with_raw(
                    "cache prune",
                    "로컬 캐시 보존 정책 적용 결과",
                    vec![
                        crate::core::output::field(
                            "dry_run",
                            "실제 삭제 없이 삭제 예정 항목만 계산했는지 여부",
                            serde_json::json!(command.dry_run),
                        ),
                        crate::core::output::field(
                            "deleted_files",
                            "삭제했거나 dry-run에서 삭제 예정인 파일 수",
                            serde_json::json!(deleted_files),
                        ),
                        crate::core::output::field(
                            "deleted_bytes",
                            "삭제했거나 dry-run에서 삭제 예정인 용량(bytes)",
                            serde_json::json!(deleted_bytes),
                        ),
                        crate::core::output::field(
                            "reports",
                            "namespace별 보존 정책과 정리 결과",
                            serde_json::json!(reports),
                        ),
                    ],
                    serde_json::json!(reports),
                )
            );
        }
    }
}

fn prune_all(dry_run: bool) -> Result<Vec<crate::core::cache::CachePruneReport>, String> {
    Ok(vec![
        kind::universe::prune_cache(dry_run)?,
        opendart::document::prune_cache(dry_run)?,
    ])
}
