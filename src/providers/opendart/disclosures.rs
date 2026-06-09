use crate::core::dotenv;
use serde::{Deserialize, Serialize};

const OPENDART_LIST_URL: &str = "https://opendart.fss.or.kr/api/list.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureQuery {
    pub corp_code: Option<String>,
    pub bgn_de: Option<String>,
    pub end_de: Option<String>,
    pub corp_cls: Option<String>,
    pub page_no: u32,
    pub page_count: u32,
}

pub fn list(query: &DisclosureQuery) -> Result<serde_json::Value, String> {
    let key = api_key()?;
    validate_query(query)?;

    let mut params = vec![
        ("crtfc_key", key),
        ("page_no", query.page_no.to_string()),
        ("page_count", query.page_count.to_string()),
    ];
    push_optional(&mut params, "corp_code", query.corp_code.as_deref());
    push_optional(&mut params, "bgn_de", query.bgn_de.as_deref());
    push_optional(&mut params, "end_de", query.end_de.as_deref());
    push_optional(&mut params, "corp_cls", query.corp_cls.as_deref());

    let url = build_url(OPENDART_LIST_URL, &params);
    let response = crate::core::http::agent()
        .get(&url)
        .header("User-Agent", "openstock/0.1")
        .call()
        .map_err(|err| format!("[OpenDART] 공시목록 요청 실패: {}", err))?;
    let status = response.status();
    let body = response
        .into_body()
        .read_to_string()
        .map_err(|err| format!("[OpenDART] 공시목록 응답 읽기 실패: {}", err))?;

    if !status.is_success() {
        return Err(format!(
            "[OpenDART] 공시목록 요청 오류 ({}): {}",
            status, body
        ));
    }

    let payload = serde_json::from_str::<serde_json::Value>(&body)
        .map_err(|err| format!("[OpenDART] 공시목록 응답 파싱 실패: {}", err))?;
    ensure_success(&payload)?;
    Ok(payload)
}

fn api_key() -> Result<String, String> {
    dotenv::read_env(&crate::core::paths::env_file())
        .get("OPENDART_API_KEY")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "OPENDART_API_KEY가 설정 파일에 없습니다: {}",
                crate::core::paths::env_file().display()
            )
        })
}

fn validate_query(query: &DisclosureQuery) -> Result<(), String> {
    if let Some(value) = &query.bgn_de {
        validate_date(value, "--from")?;
    }
    if let Some(value) = &query.end_de {
        validate_date(value, "--to")?;
    }
    if query.page_no == 0 {
        return Err("--page-no는 1 이상이어야 합니다.".to_string());
    }
    if query.page_count == 0 || query.page_count > 100 {
        return Err("--page-count는 1 이상 100 이하이어야 합니다.".to_string());
    }
    Ok(())
}

fn validate_date(value: &str, name: &str) -> Result<(), String> {
    if value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit()) {
        return Ok(());
    }
    Err(format!("{} 값은 YYYYMMDD 형식이어야 합니다.", name))
}

fn push_optional(params: &mut Vec<(&str, String)>, key: &'static str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        params.push((key, value.to_string()));
    }
}

fn build_url(base: &str, params: &[(&str, String)]) -> String {
    let query = params
        .iter()
        .map(|(key, value)| format!("{}={}", percent_encode(key), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{}?{}", base, query)
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

fn ensure_success(payload: &serde_json::Value) -> Result<(), String> {
    let status = payload
        .get("status")
        .and_then(|value| value.as_str())
        .unwrap_or("000");
    if status == "000" || status == "013" {
        return Ok(());
    }
    let message = payload
        .get("message")
        .and_then(|value| value.as_str())
        .unwrap_or("OpenDART API 실패");
    Err(format!(
        "[OpenDART] 공시목록 조회 실패 ({}): {}",
        status, message
    ))
}
