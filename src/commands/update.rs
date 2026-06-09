use crate::core::output;
use std::path::PathBuf;
use std::process::Command;

use crate::UpdateCommand;

const DEFAULT_RELEASE_API_URL: &str =
    "https://git.hananakick.cc/api/v1/repos/Autotrade/openstock/releases/latest";
const DEFAULT_ASSET_SUFFIX: &str = "-linux-x86_64.tar.gz";

pub fn handle_update(command: &UpdateCommand) {
    match update(command.force) {
        Ok(result) => {
            println!(
                "{}",
                output::explained(
                    "update",
                    "openstock 바이너리 업데이트 실행 결과",
                    vec![
                        output::field(
                            "release_api_url",
                            "최신 release 정보를 조회한 Gitea API URL",
                            serde_json::json!(result.release_api_url),
                        ),
                        output::field(
                            "current_version",
                            "현재 실행 중인 openstock 버전",
                            serde_json::json!(result.current_version),
                        ),
                        output::field(
                            "latest_version",
                            "Gitea 최신 release tag에서 해석한 버전",
                            serde_json::json!(result.latest_version),
                        ),
                        output::field(
                            "release_tag",
                            "Gitea 최신 release tag",
                            serde_json::json!(result.release_tag),
                        ),
                        output::field(
                            "release_url",
                            "Gitea release 페이지 URL",
                            serde_json::json!(result.release_url),
                        ),
                        output::field(
                            "asset_name",
                            "설치에 사용한 release asset 이름",
                            serde_json::json!(result.asset_name),
                        ),
                        output::field(
                            "asset_url",
                            "설치에 사용한 release asset 다운로드 URL",
                            serde_json::json!(result.asset_url),
                        ),
                        output::field(
                            "install_dir",
                            "업데이트 대상 바이너리를 설치한 디렉터리",
                            serde_json::json!(result.install_dir.display().to_string()),
                        ),
                        output::field(
                            "status",
                            "업데이트 명령 실행 결과",
                            serde_json::json!(result.status),
                        ),
                        output::field(
                            "stdout",
                            "release asset 설치 스크립트 표준 출력",
                            serde_json::json!(result.stdout),
                        ),
                        output::field(
                            "stderr",
                            "release asset 설치 스크립트 표준 오류 출력",
                            serde_json::json!(result.stderr),
                        ),
                    ],
                )
            );
        }
        Err(err) => eprintln!(
            "{}",
            output::error("update", "openstock 업데이트 실패", &err)
        ),
    }
}

fn update(force: bool) -> Result<UpdateResult, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let release_api_url = release_api_url();
    let release = latest_release(&release_api_url)?;
    let latest_version = normalize_version(&release.tag_name);
    let asset = release_asset(&release)?;
    let install_dir = install_dir()?;

    if !force && compare_versions(&latest_version, &current_version) != std::cmp::Ordering::Greater
    {
        return Ok(UpdateResult {
            release_api_url,
            current_version,
            latest_version,
            release_tag: release.tag_name,
            release_url: release.html_url,
            asset_name: asset.name,
            asset_url: asset.browser_download_url,
            install_dir,
            status: "up_to_date".to_string(),
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    let script = format!(
        "set -eu\n\
         tmp=\"$(mktemp -d \"${{TMPDIR:-/tmp}}/openstock-update.XXXXXX\")\"\n\
         cleanup() {{ rm -rf \"$tmp\"; }}\n\
         trap cleanup EXIT\n\
         curl -fsSL {} -o \"$tmp/openstock.tar.gz\"\n\
         tar -xzf \"$tmp/openstock.tar.gz\" -C \"$tmp\"\n\
         test -x \"$tmp/openstock\"\n\
         mkdir -p {}\n\
         target={}\n\
         tmp_target=\"{}/.openstock.tmp.$$\"\n\
         cp \"$tmp/openstock\" \"$tmp_target\"\n\
         chmod 755 \"$tmp_target\"\n\
         mv -f \"$tmp_target\" \"$target\"\n\
         printf 'installed: %s\\n' \"$target\"\n",
        shell_quote(&asset.browser_download_url),
        shell_quote(&install_dir.display().to_string()),
        shell_quote(&install_dir.join("openstock").display().to_string()),
        shell_quote(&install_dir.display().to_string()),
    );
    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .output()
        .map_err(|err| format!("release asset 설치 스크립트 실행 실패: {}", err))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!(
            "release asset 설치 스크립트가 실패했습니다. status={}, stdout={}, stderr={}",
            output.status, stdout, stderr
        ));
    }

    Ok(UpdateResult {
        release_api_url,
        current_version,
        latest_version,
        release_tag: release.tag_name,
        release_url: release.html_url,
        asset_name: asset.name,
        asset_url: asset.browser_download_url,
        install_dir,
        status: "updated".to_string(),
        stdout,
        stderr,
    })
}

fn latest_release(api_url: &str) -> Result<Release, String> {
    let response = crate::core::http::agent()
        .get(api_url)
        .call()
        .map_err(|err| format!("최신 release 조회 실패: {}", err))?;
    let status = response.status();
    let body = response
        .into_body()
        .read_to_string()
        .map_err(|err| format!("최신 release 응답 읽기 실패: {}", err))?;

    if !status.is_success() {
        return Err(format!("최신 release 조회 오류 ({}): {}", status, body));
    }

    let value = serde_json::from_str::<serde_json::Value>(&body)
        .map_err(|err| format!("최신 release 응답 파싱 실패: {} / 원문: {}", err, body))?;
    let tag_name = value
        .get("tag_name")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("최신 release 응답에 tag_name이 없습니다: {}", body))?
        .to_string();
    let html_url = value
        .get("html_url")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    let assets = value
        .get("assets")
        .and_then(|value| value.as_array())
        .ok_or_else(|| format!("최신 release 응답에 assets가 없습니다: {}", body))?
        .iter()
        .filter_map(|asset| {
            let name = asset.get("name")?.as_str()?.to_string();
            let browser_download_url = asset
                .get("browser_download_url")
                .or_else(|| asset.get("download_url"))?
                .as_str()?
                .to_string();
            Some(ReleaseAsset {
                name,
                browser_download_url,
            })
        })
        .collect::<Vec<_>>();

    Ok(Release {
        tag_name,
        html_url,
        assets,
    })
}

fn release_asset(release: &Release) -> Result<ReleaseAsset, String> {
    let suffix = std::env::var("OPENSTOCK_RELEASE_ASSET_SUFFIX")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_ASSET_SUFFIX.to_string());

    release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(&suffix) && !asset.name.ends_with(".sha256"))
        .cloned()
        .ok_or_else(|| {
            format!(
                "release {}에서 suffix '{}'에 맞는 바이너리 asset을 찾지 못했습니다",
                release.tag_name, suffix
            )
        })
}

fn install_dir() -> Result<PathBuf, String> {
    if let Some(value) = std::env::var_os("OPENSTOCK_INSTALL_DIR") {
        return Ok(PathBuf::from(value));
    }

    let exe =
        std::env::current_exe().map_err(|err| format!("현재 실행 파일 경로 확인 실패: {}", err))?;
    exe.parent().map(PathBuf::from).ok_or_else(|| {
        format!(
            "현재 실행 파일의 부모 디렉터리를 확인할 수 없습니다: {}",
            exe.display()
        )
    })
}

fn release_api_url() -> String {
    std::env::var("OPENSTOCK_RELEASE_API_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_RELEASE_API_URL.to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn normalize_version(tag: &str) -> String {
    tag.trim().trim_start_matches('v').to_string()
}

fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    let left_parts = version_parts(left);
    let right_parts = version_parts(right);
    left_parts.cmp(&right_parts)
}

fn version_parts(version: &str) -> Vec<u64> {
    version
        .split(|ch| ch == '.' || ch == '-' || ch == '+')
        .map(|part| {
            part.chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect()
}

struct Release {
    tag_name: String,
    html_url: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Clone)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

struct UpdateResult {
    release_api_url: String,
    current_version: String,
    latest_version: String,
    release_tag: String,
    release_url: String,
    asset_name: String,
    asset_url: String,
    install_dir: PathBuf,
    status: String,
    stdout: String,
    stderr: String,
}
