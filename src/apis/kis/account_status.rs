use crate::apis::kis::KisApi;
use crate::core::TraderApi;
use serde_json::json;

pub fn account_status(api: &KisApi) -> Result<String, String> {
    let account = crate::apis::kis::account_config::read_account_config()?;
    let params = [
        ("tr_id", "TTTC8434R"),
        ("CANO", account.number.as_str()),
        ("ACNT_PRDT_CD", account.product_code.as_str()),
        ("AFHR_FLPR_YN", "N"),
        ("OFL_YN", ""),
        ("INQR_DVSN", "02"),
        ("UNPR_DVSN", "01"),
        ("FUND_STTL_ICLD_YN", "N"),
        ("FNCG_AMT_AUTO_RDPT_YN", "N"),
        ("PRCS_DVSN", "00"),
        ("CTX_AREA_FK100", ""),
        ("CTX_AREA_NK100", ""),
    ];
    let response = api.call("/uapi/domestic-stock/v1/trading/inquire-balance", &params)?;
    let json: serde_json::Value = serde_json::from_str(&response)
        .map_err(|err| format!("[KIS] 계좌 상태 응답 파싱 실패: {}", err))?;

    Ok(json!({
        "broker": api.id(),
        "account": account.full_name(),
        "holdings": json.get("output1").cloned().unwrap_or_else(|| json!([])),
        "balance": json.get("output2").cloned().unwrap_or_else(|| json!([])),
        "raw": json,
    })
    .to_string())
}
