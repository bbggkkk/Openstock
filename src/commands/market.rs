use crate::apis::kis::KisApi;
use crate::core::TraderApi;

pub fn handle_market(symbol: &str) {
    let api = KisApi::new();
    match api.market(symbol) {
        Ok(json) => {
            let value = crate::core::output::parse_json_or_text(&json);
            println!(
                "{}",
                crate::core::output::explained_with_raw(
                    "market",
                    "종목 현재가와 기업 기본 정보 조회 결과",
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
                            "symbol",
                            "조회한 종목코드",
                            value
                                .get("symbol")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null),
                        ),
                        crate::core::output::field(
                            "price",
                            "현재가와 시세 관련 값",
                            value
                                .get("price")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!({})),
                        ),
                        crate::core::output::field(
                            "company",
                            "종목 및 기업 기본 정보",
                            value
                                .get("company")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!({})),
                        ),
                    ],
                    value,
                )
            );
        }
        Err(err) => eprintln!(
            "{}",
            crate::core::output::error("market", "종목 정보 조회 실패", &err)
        ),
    }
}
