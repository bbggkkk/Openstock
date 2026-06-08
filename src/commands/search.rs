use crate::providers::naver;

pub fn handle_search(query: &str) {
    match naver::search::search(query) {
        Ok(json) => {
            let value = crate::core::output::parse_json_or_text(&json);
            println!(
                "{}",
                crate::core::output::explained_with_raw(
                    "search",
                    "네이버 증권 기반 종목 이름 검색 결과",
                    vec![
                        crate::core::output::field(
                            "provider",
                            "종목 검색 데이터를 제공한 외부 서비스",
                            value
                                .get("provider")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null),
                        ),
                        crate::core::output::field(
                            "query",
                            "사용자가 입력한 검색어",
                            value
                                .get("query")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null),
                        ),
                        crate::core::output::field(
                            "stocks",
                            "검색어와 일치하는 종목 목록",
                            value
                                .get("stocks")
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
            crate::core::output::error("search", "종목 검색 실패", &err)
        ),
    }
}
