use crate::apis::kis::KisApi;
use crate::core::dotenv;

const KIS_BASE_URL: &str = "https://openapi.koreainvestment.com:9443";

pub fn call(_api: &KisApi, endpoint: &str, params: &[(&str, &str)]) -> Result<String, String> {
    let env = dotenv::read_env(&crate::core::paths::env_file());
    let token = crate::apis::kis::login::access_token_or_login()?;
    let appkey = env
        .get("KIS_APPKEY")
        .ok_or("KIS_APPKEY가 설정 파일에 없습니다.")?;
    let appsecret = env
        .get("KIS_APPSECRET")
        .ok_or("KIS_APPSECRET가 설정 파일에 없습니다.")?;
    let tr_id = params
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("tr_id"))
        .map(|(_, value)| *value);
    let query_params = params
        .iter()
        .copied()
        .filter(|(key, _)| !key.eq_ignore_ascii_case("tr_id"))
        .collect::<Vec<_>>();
    let url = build_url(endpoint, &query_params);

    let mut request = crate::core::http::agent()
        .get(&url)
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("authorization", &format!("Bearer {}", token.as_str()))
        .header("appkey", appkey)
        .header("appsecret", appsecret)
        .header("custtype", "P");

    if let Some(tr_id) = tr_id {
        request = request.header("tr_id", tr_id);
    }

    let response = request
        .call()
        .map_err(|err| format!("[KIS] API 호출 실패: {}", err))?;

    let status = response.status();
    let body = response
        .into_body()
        .read_to_string()
        .map_err(|err| format!("[KIS] API 응답 읽기 실패: {}", err))?;

    if !status.is_success() {
        return Err(format!("[KIS] API 호출 오류 ({}): {}", status, body));
    }

    Ok(body)
}

fn build_url(endpoint: &str, params: &[(&str, &str)]) -> String {
    let base = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.to_string()
    } else if endpoint.starts_with('/') {
        format!("{}{}", KIS_BASE_URL, endpoint)
    } else {
        format!("{}/{}", KIS_BASE_URL, endpoint)
    };

    if params.is_empty() {
        return base;
    }

    let query = params
        .iter()
        .map(|(key, value)| format!("{}={}", percent_encode(key), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    let separator = if base.contains('?') { "&" } else { "?" };

    format!("{}{}{}", base, separator, query)
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
