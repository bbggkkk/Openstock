use crate::apis::kis::KisApi;
use crate::core::TraderApi;

pub fn stock_info(api: &KisApi, symbol: &str) -> Result<serde_json::Value, String> {
    let params = [
        ("tr_id", "CTPF1002R"),
        ("PRDT_TYPE_CD", "300"),
        ("PDNO", symbol),
    ];
    let response = api.call(
        "/uapi/domestic-stock/v1/quotations/search-stock-info",
        &params,
    )?;
    serde_json::from_str(&response)
        .map_err(|err| format!("[KIS] 주식기본조회 응답 파싱 실패: {}", err))
}
