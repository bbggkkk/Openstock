use serde_json::json;

const NAVER_STOCK_SEARCH_URL: &str = "https://m.stock.naver.com/front-api/search/autoComplete";
const NAVER_STOCK_BASE_URL: &str = "https://m.stock.naver.com";

pub fn search(query: &str) -> Result<String, String> {
    let query = query.trim();
    if query.is_empty() {
        return Err("검색어가 비어 있습니다.".to_string());
    }

    let url = format!(
        "{}?query={}&target=stock",
        NAVER_STOCK_SEARCH_URL,
        percent_encode(query)
    );

    let response = ureq::get(&url)
        .header("User-Agent", "openstock/0.1")
        .call()
        .map_err(|err| format!("[Naver] 종목 검색 요청 실패: {}", err))?;
    let status = response.status();
    let body = response
        .into_body()
        .read_to_string()
        .map_err(|err| format!("[Naver] 종목 검색 응답 읽기 실패: {}", err))?;

    if !status.is_success() {
        return Err(format!("[Naver] 종목 검색 오류 ({}): {}", status, body));
    }

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|err| format!("[Naver] 종목 검색 응답 파싱 실패: {}", err))?;
    let stocks = parse_stocks(&json);

    Ok(json!({
        "provider": "NAVER",
        "query": query,
        "stocks": stocks,
        "raw": json,
    })
    .to_string())
}

fn parse_stocks(value: &serde_json::Value) -> Vec<serde_json::Value> {
    let modern_items = value
        .get("result")
        .and_then(|result| result.get("items"))
        .and_then(|items| items.as_array())
        .into_iter()
        .flatten();

    let legacy_items = value
        .get("items")
        .and_then(|items| items.as_array())
        .into_iter()
        .flatten()
        .flat_map(|group| group.as_array().into_iter().flatten());

    modern_items
        .chain(legacy_items)
        .filter_map(parse_stock_item)
        .collect()
}

fn parse_stock_item(item: &serde_json::Value) -> Option<serde_json::Value> {
    if let Some(values) = item.as_array() {
        let name = values
            .first()
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let code = values.get(1).and_then(|value| value.as_str()).unwrap_or("");
        let market = values.get(2).and_then(|value| value.as_str()).unwrap_or("");

        if code.is_empty() {
            return None;
        }

        return Some(json!({
            "code": code,
            "name": strip_html(name),
            "market": market,
        }));
    }

    if let Some(object) = item.as_object() {
        let code = object
            .get("code")
            .or_else(|| object.get("itemCode"))
            .or_else(|| object.get("symbolCode"))
            .or_else(|| object.get("ticker"))
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let name = object
            .get("name")
            .or_else(|| object.get("stockName"))
            .or_else(|| object.get("itemName"))
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let market = object
            .get("typeName")
            .or_else(|| object.get("stockExchangeName"))
            .or_else(|| object.get("marketName"))
            .or_else(|| object.get("typeCode"))
            .or_else(|| object.get("market"))
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let market_code = object
            .get("typeCode")
            .or_else(|| object.get("stockExchangeType"))
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let url = object
            .get("url")
            .or_else(|| object.get("endUrl"))
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let reuters_code = object
            .get("reutersCode")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let nation_code = object
            .get("nationCode")
            .or_else(|| object.get("nationType"))
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let category = object
            .get("category")
            .or_else(|| object.get("stockType"))
            .and_then(|value| value.as_str())
            .unwrap_or("");

        if code.is_empty() {
            return None;
        }

        return Some(json!({
            "code": code,
            "name": strip_html(name),
            "market": market,
            "market_code": market_code,
            "nation_code": nation_code,
            "category": category,
            "reuters_code": reuters_code,
            "url": full_naver_url(url),
        }));
    }

    None
}

fn full_naver_url(url: &str) -> String {
    if url.is_empty() || url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }

    format!("{}{}", NAVER_STOCK_BASE_URL, url)
}

fn strip_html(value: &str) -> String {
    let mut result = String::new();
    let mut inside_tag = false;

    for ch in value.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(ch),
            _ => {}
        }
    }

    result
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}
