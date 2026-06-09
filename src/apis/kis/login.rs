use crate::core::{dotenv, LoginArguments};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const KIS_BASE_URL: &str = "https://openapi.koreainvestment.com:9443";
const KIS_TOKEN_PATH: &str = "/oauth2/tokenP";
const KIS_ACCESS_TOKEN_EXPIRED_AT_KEY: &str = "KIS_ACCESS_TOKEN_EXPIRED_AT";

/// 한국투자증권 실전 API 로그인에 필요한 인자
pub struct KisLoginArguments {
    pub appkey: String,
    pub appsecret: String,
    pub force: bool,
}

impl KisLoginArguments {
    pub fn new(appkey: String, appsecret: String, force: bool) -> Self {
        Self {
            appkey,
            appsecret,
            force,
        }
    }
}

impl LoginArguments for KisLoginArguments {
    fn token_env_key(&self) -> &'static str {
        "KIS_ACCESS_TOKEN"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// KIS 실전 REST 접근토큰을 발급하고 설정 파일에 저장한다.
pub fn login(args: &KisLoginArguments) -> Result<String, String> {
    let env_path = crate::core::paths::env_file();
    let env = dotenv::read_env(&env_path);
    if !args.force {
        if let Some(access_token) = valid_access_token(&env) {
            return Ok(access_token.to_string());
        }
    }

    validate_credentials(args)?;

    let url = format!("{}{}", KIS_BASE_URL, KIS_TOKEN_PATH);
    let request_body = serde_json::json!({
        "grant_type": "client_credentials",
        "appkey": args.appkey,
        "appsecret": args.appsecret,
    });

    let response = crate::core::http::agent()
        .post(&url)
        .header("Content-Type", "application/json; charset=UTF-8")
        .send_json(&request_body)
        .map_err(|err| format!("[KIS] 접근토큰 발급 요청 실패: {}", err))?;

    let status = response.status();
    let response_body = response
        .into_body()
        .read_to_string()
        .map_err(|err| format!("[KIS] 접근토큰 응답 읽기 실패: {}", err))?;

    let json = serde_json::from_str::<serde_json::Value>(&response_body).map_err(|err| {
        format!(
            "[KIS] 접근토큰 응답 파싱 실패: {} / 원문: {}",
            err, response_body
        )
    })?;

    if !status.is_success() {
        return Err(format!(
            "[KIS] 접근토큰 발급 오류 ({}): {}",
            status,
            kis_error_message(&json, &response_body)
        ));
    }

    let access_token = json
        .get("access_token")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("[KIS] 응답에 access_token이 없습니다: {}", response_body))?
        .to_string();

    let token_type = json
        .get("token_type")
        .and_then(|value| value.as_str())
        .unwrap_or("Bearer");
    let expires_at = json
        .get("access_token_token_expired")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let expires_in = json
        .get("expires_in")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);

    dotenv::write_env(&env_path, "KIS_APPKEY", &args.appkey)?;
    dotenv::write_env(&env_path, "KIS_APPSECRET", &args.appsecret)?;
    dotenv::write_env(&env_path, args.token_env_key(), &access_token)?;
    dotenv::write_env(&env_path, "KIS_TOKEN_TYPE", token_type)?;
    if !expires_at.is_empty() {
        dotenv::write_env(&env_path, KIS_ACCESS_TOKEN_EXPIRED_AT_KEY, expires_at)?;
    }
    if expires_in > 0 {
        dotenv::write_env(
            &env_path,
            "KIS_ACCESS_TOKEN_EXPIRES_IN",
            &expires_in.to_string(),
        )?;
    }

    Ok(access_token)
}

pub(crate) fn access_token_or_login() -> Result<String, String> {
    let env_path = crate::core::paths::env_file();
    let env = dotenv::read_env(&env_path);
    if let Some(access_token) = valid_access_token(&env) {
        return Ok(access_token.to_string());
    }

    let appkey = env
        .get("KIS_APPKEY")
        .ok_or("KIS_APPKEY가 설정 파일에 없습니다.")?
        .to_string();
    let appsecret = env
        .get("KIS_APPSECRET")
        .ok_or("KIS_APPSECRET가 설정 파일에 없습니다.")?
        .to_string();

    login(&KisLoginArguments::new(appkey, appsecret, true))
}

fn validate_credentials(args: &KisLoginArguments) -> Result<(), String> {
    if args.appkey.trim().is_empty() {
        return Err("[KIS] KIS_APPKEY가 비어 있습니다".to_string());
    }
    if args.appsecret.trim().is_empty() {
        return Err("[KIS] KIS_APPSECRET가 비어 있습니다".to_string());
    }
    Ok(())
}

pub(crate) fn valid_access_token(env: &HashMap<String, String>) -> Option<&str> {
    let access_token = env.get("KIS_ACCESS_TOKEN")?;
    if access_token.trim().is_empty() {
        return None;
    }

    let expires_at = env.get(KIS_ACCESS_TOKEN_EXPIRED_AT_KEY)?;
    if expires_at.trim().is_empty() || !is_future_kst_datetime(expires_at) {
        return None;
    }

    Some(access_token.as_str())
}

fn is_future_kst_datetime(datetime: &str) -> bool {
    let now_kst = current_kst_datetime_string();
    datetime.trim() > now_kst.as_str()
}

fn current_kst_datetime_string() -> String {
    let now_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let kst_seconds = now_seconds + 9 * 60 * 60;
    format_unix_datetime(kst_seconds)
}

fn format_unix_datetime(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, second
    )
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };

    (year, month, day)
}

fn kis_error_message(json: &serde_json::Value, fallback: &str) -> String {
    json.get("msg1")
        .or_else(|| json.get("msg"))
        .or_else(|| json.get("error_description"))
        .or_else(|| json.get("error"))
        .and_then(|value| value.as_str())
        .unwrap_or(fallback)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_unix_datetime() {
        assert_eq!(format_unix_datetime(0), "1970-01-01 00:00:00");
        assert_eq!(format_unix_datetime(1_704_067_200), "2024-01-01 00:00:00");
    }

    #[test]
    fn returns_token_only_when_expiration_exists() {
        let mut env = HashMap::new();
        env.insert("KIS_ACCESS_TOKEN".to_string(), "token".to_string());
        assert_eq!(valid_access_token(&env), None);
    }

    #[test]
    fn returns_token_when_expiration_is_future() {
        let mut env = HashMap::new();
        env.insert("KIS_ACCESS_TOKEN".to_string(), "token".to_string());
        env.insert(
            KIS_ACCESS_TOKEN_EXPIRED_AT_KEY.to_string(),
            "9999-12-31 23:59:59".to_string(),
        );
        assert_eq!(valid_access_token(&env), Some("token"));
    }
}
