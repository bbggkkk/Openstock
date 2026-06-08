use crate::apis::kis::stock_info;
use crate::apis::kis::KisApi;
use crate::core::TraderApi;
use serde_json::json;

pub fn market(api: &KisApi, symbol: &str) -> Result<String, String> {
    let symbol = symbol.trim();
    if !is_stock_code(symbol) {
        return Err("종목코드는 6자리 숫자여야 합니다. 예: 005930".to_string());
    }

    let price = inquire_price(api, symbol)?;
    let company = stock_info::stock_info(api, symbol)?;

    Ok(json!({
        "broker": api.id(),
        "symbol": symbol,
        "price": price.get("output").cloned().unwrap_or_else(|| json!({})),
        "company": company.get("output").cloned().unwrap_or_else(|| json!({})),
        "raw": {
            "price": price,
            "company": company,
        },
    })
    .to_string())
}

fn inquire_price(api: &KisApi, symbol: &str) -> Result<serde_json::Value, String> {
    let params = [
        ("tr_id", "FHKST01010100"),
        ("FID_COND_MRKT_DIV_CODE", "J"),
        ("FID_INPUT_ISCD", symbol),
    ];
    let response = api.call("/uapi/domestic-stock/v1/quotations/inquire-price", &params)?;
    serde_json::from_str(&response).map_err(|err| format!("[KIS] 현재가 응답 파싱 실패: {}", err))
}

fn is_stock_code(value: &str) -> bool {
    value.len() == 6 && value.chars().all(|ch| ch.is_ascii_digit())
}
