use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum ScoreCommands {
    /// 종목 평가 점수 저장 또는 갱신
    Set(ScoreSetCommand),

    /// 종목 평가 점수 조회
    Get(ScoreSymbolCommand),

    /// 저장된 종목 평가 점수 목록 조회
    List,

    /// 종목 평가 점수 삭제
    Delete(ScoreSymbolCommand),
}

#[derive(Args)]
pub struct ScoreSetCommand {
    /// 종목코드 또는 종목 ID
    pub symbol: String,

    /// 종목 평가 점수 (0~100)
    pub score: u16,
}

#[derive(Args)]
pub struct ScoreSymbolCommand {
    /// 종목코드 또는 종목 ID
    pub symbol: String,
}

pub fn handle_score(sub: &ScoreCommands) {
    match sub {
        ScoreCommands::Set(command) => {
            match crate::core::scores::set(&command.symbol, command.score) {
                Ok(record) => print_record("score set", "종목 평가 점수 저장 결과", record),
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error("score set", "종목 평가 점수 저장 실패", &err)
                ),
            }
        }
        ScoreCommands::Get(command) => match crate::core::scores::get(&command.symbol) {
            Ok(Some(record)) => print_record("score get", "종목 평가 점수 조회 결과", record),
            Ok(None) => eprintln!(
                "{}",
                crate::core::output::error(
                    "score get",
                    "종목 평가 점수 조회 실패",
                    "해당 종목의 저장된 점수가 없습니다.",
                )
            ),
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("score get", "종목 평가 점수 조회 실패", &err)
            ),
        },
        ScoreCommands::List => match crate::core::scores::list() {
            Ok(scores) => println!(
                "{}",
                crate::core::output::explained_with_raw(
                    "score list",
                    "저장된 종목 평가 점수 목록",
                    vec![
                        crate::core::output::field(
                            "path",
                            "종목 평가 점수를 저장하는 파일 경로",
                            serde_json::json!(crate::core::scores::path()),
                        ),
                        crate::core::output::field(
                            "count",
                            "저장된 종목 평가 점수 개수",
                            serde_json::json!(scores.len()),
                        ),
                        crate::core::output::field(
                            "scores",
                            "점수 내림차순으로 정렬한 종목 평가 목록",
                            serde_json::json!(scores),
                        ),
                    ],
                    serde_json::Value::Null,
                )
            ),
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("score list", "종목 평가 점수 목록 조회 실패", &err)
            ),
        },
        ScoreCommands::Delete(command) => match crate::core::scores::delete(&command.symbol) {
            Ok(removed) => println!(
                "{}",
                crate::core::output::explained_with_raw(
                    "score delete",
                    "종목 평가 점수 삭제 결과",
                    vec![
                        crate::core::output::field(
                            "path",
                            "종목 평가 점수를 저장하는 파일 경로",
                            serde_json::json!(crate::core::scores::path()),
                        ),
                        crate::core::output::field(
                            "symbol",
                            "삭제 요청한 종목코드 또는 종목 ID",
                            serde_json::json!(command.symbol.trim().to_ascii_uppercase()),
                        ),
                        crate::core::output::field(
                            "deleted",
                            "저장된 점수가 존재해서 삭제되었는지 여부",
                            serde_json::json!(removed.is_some()),
                        ),
                        crate::core::output::field(
                            "removed",
                            "삭제된 기존 점수 기록",
                            serde_json::json!(removed),
                        ),
                    ],
                    serde_json::Value::Null,
                )
            ),
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("score delete", "종목 평가 점수 삭제 실패", &err)
            ),
        },
    }
}

fn print_record(command: &str, description: &str, record: crate::core::scores::StockScore) {
    println!(
        "{}",
        crate::core::output::explained_with_raw(
            command,
            description,
            vec![
                crate::core::output::field(
                    "path",
                    "종목 평가 점수를 저장하는 파일 경로",
                    serde_json::json!(crate::core::scores::path()),
                ),
                crate::core::output::field(
                    "symbol",
                    "점수를 매긴 종목코드 또는 종목 ID",
                    serde_json::json!(record.symbol),
                ),
                crate::core::output::field(
                    "score",
                    "종목 평가 점수. 0은 최저, 100은 최고",
                    serde_json::json!(record.score),
                ),
                crate::core::output::field(
                    "updated_at_unix",
                    "점수를 저장하거나 갱신한 Unix timestamp(초)",
                    serde_json::json!(record.updated_at_unix),
                ),
            ],
            serde_json::Value::Null,
        )
    );
}
