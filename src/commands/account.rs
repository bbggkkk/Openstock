use crate::apis::kis::KisApi;
use crate::core::TraderApi;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum AccountCommands {
    /// 계좌 상태 조회 (잔액, 보유종목)
    Status,
}

pub fn handle_account(sub: &AccountCommands) {
    match sub {
        AccountCommands::Status => {
            let api = KisApi::new();
            match api.account_status() {
                Ok(json) => {
                    let value = crate::core::output::parse_json_or_text(&json);
                    println!(
                        "{}",
                        crate::core::output::explained_with_raw(
                            "account status",
                            "계좌 잔액과 보유종목 조회 결과",
                            vec![
                                crate::core::output::field(
                                    "broker",
                                    "조회에 사용한 증권사 API",
                                    value
                                        .get("broker")
                                        .cloned()
                                        .unwrap_or(serde_json::Value::Null),
                                ),
                                crate::core::output::field(
                                    "account",
                                    "조회한 계좌번호",
                                    value
                                        .get("account")
                                        .cloned()
                                        .unwrap_or(serde_json::Value::Null),
                                ),
                                crate::core::output::field(
                                    "balance",
                                    "예수금, 총평가금액, 손익 등 계좌 요약",
                                    value
                                        .get("balance")
                                        .cloned()
                                        .unwrap_or_else(|| serde_json::json!([])),
                                ),
                                crate::core::output::field(
                                    "holdings",
                                    "현재 보유 중인 종목 목록",
                                    value
                                        .get("holdings")
                                        .cloned()
                                        .unwrap_or_else(|| serde_json::json!([])),
                                ),
                            ],
                            value,
                        )
                    );
                }
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error("account status", "계좌 상태 조회 실패", &err)
                ),
            }
        }
    }
}
