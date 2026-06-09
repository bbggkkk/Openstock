use crate::apis::kis::KisApi;
use crate::core::TraderApi;
use serde_json::json;

const HISTORY_ENDPOINT: &str = "/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice";

pub fn market_history(
    api: &KisApi,
    symbol: &str,
    start_date: &str,
    end_date: &str,
    period: &str,
    adjusted: bool,
) -> Result<String, String> {
    let symbol = symbol.trim();
    let start_date = start_date.trim();
    let end_date = end_date.trim();
    let period = normalize_period(period)?;
    validate_symbol(symbol)?;
    validate_date(start_date, "--from")?;
    validate_date(end_date, "--to")?;
    if start_date > end_date {
        return Err("--from은 --to보다 늦을 수 없습니다.".to_string());
    }

    let adj_price_code = if adjusted { "0" } else { "1" };
    let params = [
        ("tr_id", "FHKST03010100"),
        ("FID_COND_MRKT_DIV_CODE", "J"),
        ("FID_INPUT_ISCD", symbol),
        ("FID_INPUT_DATE_1", start_date),
        ("FID_INPUT_DATE_2", end_date),
        ("FID_PERIOD_DIV_CODE", period),
        ("FID_ORG_ADJ_PRC", adj_price_code),
    ];
    let response = api.call(HISTORY_ENDPOINT, &params)?;
    let value = serde_json::from_str::<serde_json::Value>(&response)
        .map_err(|err| format!("[KIS] 기간별 시세 응답 파싱 실패: {}", err))?;
    ensure_success(&value)?;

    let mut candles = value
        .get("output2")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .map(normalize_candle)
        .collect::<Vec<_>>();
    candles.sort_by(|a, b| {
        a.get("date")
            .and_then(|value| value.as_str())
            .cmp(&b.get("date").and_then(|value| value.as_str()))
    });

    Ok(json!({
        "broker": api.id(),
        "symbol": symbol,
        "period": period,
        "start_date": start_date,
        "end_date": end_date,
        "adjusted": adjusted,
        "count": candles.len(),
        "summary": value.get("output1").cloned().unwrap_or_else(|| json!({})),
        "candles": candles,
        "raw": value,
    })
    .to_string())
}

fn normalize_candle(item: &serde_json::Value) -> serde_json::Value {
    json!({
        "date": text(item, "stck_bsop_date"),
        "open": text(item, "stck_oprc"),
        "high": text(item, "stck_hgpr"),
        "low": text(item, "stck_lwpr"),
        "close": text(item, "stck_clpr"),
        "volume": text(item, "acml_vol"),
        "trading_value": text(item, "acml_tr_pbmn"),
        "change": text(item, "prdy_vrss"),
        "change_sign": text(item, "prdy_vrss_sign"),
        "change_rate": text(item, "prdy_ctrt"),
    })
}

fn text(item: &serde_json::Value, key: &str) -> String {
    item.get(key)
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
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
        .unwrap_or("KIS 기간별 시세 조회 실패");
    Err(format!("[KIS] 기간별 시세 조회 실패: {}", message))
}

fn normalize_period(value: &str) -> Result<&'static str, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "D" | "DAY" | "DAILY" => Ok("D"),
        "W" | "WEEK" | "WEEKLY" => Ok("W"),
        "M" | "MONTH" | "MONTHLY" => Ok("M"),
        "Y" | "YEAR" | "YEARLY" => Ok("Y"),
        _ => Err("--period는 D, W, M, Y 중 하나여야 합니다.".to_string()),
    }
}

fn validate_symbol(value: &str) -> Result<(), String> {
    if value.len() == 6 && value.chars().all(|ch| ch.is_ascii_digit()) {
        return Ok(());
    }
    Err("종목코드는 6자리 숫자여야 합니다. 예: 005930".to_string())
}

fn validate_date(value: &str, name: &str) -> Result<(), String> {
    if value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit()) {
        return Ok(());
    }
    Err(format!("{} 값은 YYYYMMDD 형식이어야 합니다.", name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_period_aliases() {
        assert_eq!(normalize_period("d").unwrap(), "D");
        assert_eq!(normalize_period("weekly").unwrap(), "W");
        assert_eq!(normalize_period("M").unwrap(), "M");
        assert!(normalize_period("hour").is_err());
    }

    #[test]
    fn normalizes_candle_fields() {
        let item = json!({
            "stck_bsop_date": "20260609",
            "stck_oprc": "70000",
            "stck_hgpr": "71000",
            "stck_lwpr": "69000",
            "stck_clpr": "70500",
            "acml_vol": "123456",
            "acml_tr_pbmn": "8000000000",
            "prdy_vrss": "500",
            "prdy_vrss_sign": "2",
            "prdy_ctrt": "0.71"
        });

        let candle = normalize_candle(&item);
        assert_eq!(candle["date"], "20260609");
        assert_eq!(candle["open"], "70000");
        assert_eq!(candle["close"], "70500");
        assert_eq!(candle["volume"], "123456");
    }
}
