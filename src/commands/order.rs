use crate::apis::kis::KisApi;
use crate::core::TraderApi;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum OrderCommands {
    /// 국내주식 매수 주문
    Buy(OrderPlaceCommand),

    /// 국내주식 매도 주문
    Sell(OrderPlaceCommand),

    /// 주문/체결 조회
    Status(OrderStatusCommand),
}

#[derive(Args)]
pub struct OrderPlaceCommand {
    /// 종목코드
    pub symbol: String,

    /// 주문 수량
    #[arg(long)]
    pub qty: u64,

    /// 지정가
    #[arg(long, conflicts_with = "market")]
    pub price: Option<u64>,

    /// 시장가 주문
    #[arg(long)]
    pub market: bool,
}

#[derive(Args)]
pub struct OrderStatusCommand {
    /// 주문번호
    pub order_no: Option<String>,

    /// 조회 시작일 (YYYYMMDD)
    #[arg(long = "from")]
    pub from: Option<String>,

    /// 조회 종료일 (YYYYMMDD)
    #[arg(long = "to")]
    pub to: Option<String>,
}

pub fn handle_order(sub: &OrderCommands) {
    let api = KisApi::new();
    match sub {
        OrderCommands::Buy(command) => {
            match api.buy(&command.symbol, command.qty, command.price, command.market) {
                Ok(json) => print_order_result("order buy", "국내주식 매수 주문 결과", &json),
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error("order buy", "매수 주문 실패", &err)
                ),
            }
        }
        OrderCommands::Sell(command) => {
            match api.sell(&command.symbol, command.qty, command.price, command.market) {
                Ok(json) => print_order_result("order sell", "국내주식 매도 주문 결과", &json),
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error("order sell", "매도 주문 실패", &err)
                ),
            }
        }
        OrderCommands::Status(command) => {
            match api.order_status(
                command.order_no.as_deref(),
                command.from.as_deref(),
                command.to.as_deref(),
            ) {
                Ok(json) => print_order_status_result(&json),
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error("order status", "주문 조회 실패", &err)
                ),
            }
        }
    }
}

fn print_order_result(command: &str, description: &str, json: &str) {
    let value = crate::core::output::parse_json_or_text(json);
    println!(
        "{}",
        crate::core::output::explained_with_raw(
            command,
            description,
            vec![
                crate::core::output::field(
                    "broker",
                    "주문에 사용한 증권사 API",
                    value
                        .get("broker")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "side",
                    "주문 방향. buy는 매수, sell은 매도",
                    value
                        .get("side")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "symbol",
                    "주문한 종목코드",
                    value
                        .get("symbol")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "qty",
                    "주문 수량",
                    value.get("qty").cloned().unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "order_type",
                    "주문 유형. limit은 지정가, market은 시장가",
                    value
                        .get("order_type")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "price",
                    "지정가 주문 가격. 시장가 주문이면 null",
                    value
                        .get("price")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "order",
                    "증권사에서 반환한 주문 접수 정보",
                    value
                        .get("order")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({})),
                ),
            ],
            serde_json::Value::Null,
        )
    );
}

fn print_order_status_result(json: &str) {
    let value = crate::core::output::parse_json_or_text(json);
    println!(
        "{}",
        crate::core::output::explained_with_raw(
            "order status",
            "주문 및 체결 내역 조회 결과",
            vec![
                crate::core::output::field(
                    "broker",
                    "조회에 사용한 증권사 API",
                    value
                        .get("broker")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "account",
                    "조회한 계좌번호",
                    value
                        .get("account")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "order_no",
                    "조회 대상으로 지정한 주문번호. 없으면 기간 내 전체 주문",
                    value
                        .get("order_no")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "orders",
                    "주문 및 체결 상세 목록",
                    value
                        .get("orders")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([])),
                ),
                crate::core::output::field(
                    "summary",
                    "주문 조회 요약 정보",
                    value
                        .get("summary")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({})),
                ),
            ],
            serde_json::Value::Null,
        )
    );
}
