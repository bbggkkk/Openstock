use crate::core::output;
use std::path::PathBuf;
use std::process::Command;

const DEFAULT_INSTALL_SCRIPT_URL: &str =
    "https://git.hananakick.cc/Autotrade/openstock/raw/branch/main/scripts/install.sh";

pub fn handle_update() {
    match update() {
        Ok(result) => {
            println!(
                "{}",
                output::explained(
                    "update",
                    "openstock 바이너리 업데이트 실행 결과",
                    vec![
                        output::field(
                            "installer_url",
                            "업데이트에 사용한 원격 설치 스크립트 URL",
                            serde_json::json!(result.installer_url),
                        ),
                        output::field(
                            "install_dir",
                            "업데이트 대상 바이너리를 설치한 디렉터리",
                            serde_json::json!(result.install_dir.display().to_string()),
                        ),
                        output::field(
                            "status",
                            "업데이트 명령 실행 결과",
                            serde_json::json!("updated"),
                        ),
                        output::field(
                            "stdout",
                            "설치 스크립트 표준 출력",
                            serde_json::json!(result.stdout),
                        ),
                        output::field(
                            "stderr",
                            "설치 스크립트 표준 오류 출력",
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

fn update() -> Result<UpdateResult, String> {
    let install_dir = install_dir()?;
    let installer_url = installer_url();
    let command = format!("curl -fsSL {} | sh", shell_quote(&installer_url));
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("OPENSTOCK_INSTALL_DIR", &install_dir)
        .output()
        .map_err(|err| format!("업데이트 스크립트 실행 실패: {}", err))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!(
            "업데이트 스크립트가 실패했습니다. status={}, stdout={}, stderr={}",
            output.status, stdout, stderr
        ));
    }

    Ok(UpdateResult {
        installer_url,
        install_dir,
        stdout,
        stderr,
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

fn installer_url() -> String {
    std::env::var("OPENSTOCK_INSTALL_SCRIPT_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_INSTALL_SCRIPT_URL.to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

struct UpdateResult {
    installer_url: String,
    install_dir: PathBuf,
    stdout: String,
    stderr: String,
}
