use crate::apis::kis::KisApi;
use crate::core::dotenv;
use crate::core::TraderApi;
use serde_json::json;

const KIS_BASE_URL: &str = "https://openapi.koreainvestment.com:9443";
const ORDER_CASH_ENDPOINT: &str = "/uapi/domestic-stock/v1/trading/order-cash";

pub(crate) enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    fn tr_id(&self) -> &'static str {
        match self {
            OrderSide::Buy => "TTTC0802U",
            OrderSide::Sell => "TTTC0801U",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        }
    }
}

pub(crate) fn order_cash(
    api: &KisApi,
    side: OrderSide,
    symbol: &str,
    qty: u64,
    price: Option<u64>,
    market: bool,
) -> Result<String, String> {
    validate_order(symbol, qty, price, market)?;

    let account = crate::apis::kis::account_config::read_account_config()?;
    let order_type = if market { "01" } else { "00" };
    let order_price = if market {
        "0".to_string()
    } else {
        price.unwrap_or(0).to_string()
    };
    let body = json!({
        "CANO": account.number,
        "ACNT_PRDT_CD": account.product_code,
        "PDNO": symbol,
        "ORD_DVSN": order_type,
        "ORD_QTY": qty.to_string(),
        "ORD_UNPR": order_price,
    });
    let raw = post_order(api, side.tr_id(), &body)?;
    let value = serde_json::from_str::<serde_json::Value>(&raw)
        .map_err(|err| format!("[KIS] 주문 응답 파싱 실패: {}", err))?;
    ensure_success(&value)?;

    Ok(json!({
        "broker": api.id(),
        "side": side.name(),
        "symbol": symbol,
        "qty": qty,
        "order_type": if market { "market" } else { "limit" },
        "price": if market { serde_json::Value::Null } else { json!(price.unwrap_or(0)) },
        "order": value.get("output").cloned().unwrap_or_else(|| json!({})),
        "raw": value,
    })
    .to_string())
}

fn post_order(api: &KisApi, tr_id: &str, body: &serde_json::Value) -> Result<String, String> {
    let env = dotenv::read_env(&crate::core::paths::env_file());
    let token = api
        .token()
        .or_else(|| env.get("KIS_ACCESS_TOKEN").map(String::as_str))
        .ok_or("로그인이 필요합니다. `openstock api login`을 먼저 실행하세요.")?;
    let appkey = env
        .get("KIS_APPKEY")
        .ok_or("KIS_APPKEY가 설정 파일에 없습니다.")?;
    let appsecret = env
        .get("KIS_APPSECRET")
        .ok_or("KIS_APPSECRET가 설정 파일에 없습니다.")?;
    let url = format!("{}{}", KIS_BASE_URL, ORDER_CASH_ENDPOINT);

    let response = crate::core::http::agent()
        .post(&url)
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("authorization", &format!("Bearer {}", token))
        .header("appkey", appkey)
        .header("appsecret", appsecret)
        .header("tr_id", tr_id)
        .header("custtype", "P")
        .send_json(body)
        .map_err(|err| format!("[KIS] 주문 요청 실패: {}", err))?;
    let status = response.status();
    let response_body = response
        .into_body()
        .read_to_string()
        .map_err(|err| format!("[KIS] 주문 응답 읽기 실패: {}", err))?;

    if !status.is_success() {
        return Err(format!("[KIS] 주문 오류 ({}): {}", status, response_body));
    }

    Ok(response_body)
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
        .unwrap_or("KIS 주문 실패");
    Err(format!("[KIS] 주문 실패: {}", message))
}

fn validate_order(symbol: &str, qty: u64, price: Option<u64>, market: bool) -> Result<(), String> {
    if symbol.len() != 6 || !symbol.chars().all(|ch| ch.is_ascii_digit()) {
        return Err("종목코드는 6자리 숫자여야 합니다. 예: 005930".to_string());
    }
    if qty == 0 {
        return Err("주문 수량은 1 이상이어야 합니다.".to_string());
    }
    if market {
        return Ok(());
    }
    if price.unwrap_or(0) == 0 {
        return Err(
            "지정가 주문은 --price 값이 필요합니다. 시장가는 --market을 사용하세요.".to_string(),
        );
    }
    Ok(())
}
