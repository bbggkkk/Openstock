use crate::apis::kis::account_config;
use crate::apis::kis::KisApi;
use crate::core::TraderApi;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn order_status(
    api: &KisApi,
    order_no: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<String, String> {
    let account = account_config::read_account_config()?;
    let today = current_kst_date_string();
    let start_date = start_date.unwrap_or(today.as_str());
    let end_date = end_date.unwrap_or(today.as_str());
    validate_date(start_date, "--from")?;
    validate_date(end_date, "--to")?;
    let order_no = order_no.unwrap_or("").trim();
    let params = [
        ("tr_id", "TTTC8001R"),
        ("CANO", account.number.as_str()),
        ("ACNT_PRDT_CD", account.product_code.as_str()),
        ("INQR_STRT_DT", start_date),
        ("INQR_END_DT", end_date),
        ("SLL_BUY_DVSN_CD", "00"),
        ("INQR_DVSN", "00"),
        ("PDNO", ""),
        ("CCLD_DVSN", "00"),
        ("ORD_GNO_BRNO", ""),
        ("ODNO", order_no),
        ("INQR_DVSN_3", "00"),
        ("INQR_DVSN_1", ""),
        ("CTX_AREA_FK100", ""),
        ("CTX_AREA_NK100", ""),
    ];
    let response = api.call(
        "/uapi/domestic-stock/v1/trading/inquire-daily-ccld",
        &params,
    )?;
    let value = serde_json::from_str::<serde_json::Value>(&response)
        .map_err(|err| format!("[KIS] 주문 조회 응답 파싱 실패: {}", err))?;
    ensure_success(&value)?;
    let orders = value.get("output1").cloned().unwrap_or_else(|| json!([]));

    Ok(json!({
        "broker": api.id(),
        "account": account.full_name(),
        "order_no": if order_no.is_empty() { serde_json::Value::Null } else { json!(order_no) },
        "from": start_date,
        "to": end_date,
        "orders": orders,
        "summary": value.get("output2").cloned().unwrap_or_else(|| json!({})),
        "raw": value,
    })
    .to_string())
}

fn validate_date(value: &str, name: &str) -> Result<(), String> {
    if value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit()) {
        return Ok(());
    }

    Err(format!("{} 값은 YYYYMMDD 형식이어야 합니다.", name))
}

fn ensure_success(value: &serde_json::Value) -> Result<(), String> {
    if value
        .get("rt_cd")
        .and_then(|value| value.as_str())
        .unwrap_or("0")
        == "0"
    {
        return Ok(());
    }

    let message = value
        .get("msg1")
        .or_else(|| value.get("msg"))
        .and_then(|value| value.as_str())
        .unwrap_or("KIS 주문 조회 실패");
    Err(format!("[KIS] 주문 조회 실패: {}", message))
}

fn current_kst_date_string() -> String {
    let now_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let kst_seconds = now_seconds + 9 * 60 * 60;
    let days = kst_seconds.div_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!("{:04}{:02}{:02}", year, month, day)
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
