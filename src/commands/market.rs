use crate::apis::kis::KisApi;
use crate::core::TraderApi;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct MarketCommand {
    /// 종목코드. 하위 명령 없이 지정하면 현재가와 기업 기본 정보를 조회
    pub symbol: Option<String>,

    #[command(subcommand)]
    pub sub: Option<MarketCommands>,
}

#[derive(Subcommand)]
pub enum MarketCommands {
    /// 기간별 OHLCV 가격 히스토리 조회
    History(MarketHistoryCommand),
}

#[derive(Args)]
pub struct MarketHistoryCommand {
    /// 종목코드
    pub symbol: String,

    /// 조회 시작일 YYYYMMDD
    #[arg(long = "from")]
    pub from: String,

    /// 조회 종료일 YYYYMMDD
    #[arg(long = "to")]
    pub to: String,

    /// 기간 구분: D=일봉, W=주봉, M=월봉, Y=년봉
    #[arg(long, default_value = "D")]
    pub period: String,

    /// 원주가를 조회. 기본값은 수정주가
    #[arg(long)]
    pub raw_price: bool,
}

pub fn handle_market(command: &MarketCommand) {
    match &command.sub {
        Some(MarketCommands::History(history)) => handle_market_history(history),
        None => match command.symbol.as_deref() {
            Some(symbol) => handle_market_overview(symbol),
            None => eprintln!(
                "{}",
                crate::core::output::error(
                    "market",
                    "종목 정보 조회 실패",
                    "종목코드를 지정하거나 `market history` 하위 명령을 사용하세요.",
                )
            ),
        },
    }
}

fn handle_market_overview(symbol: &str) {
    let api = KisApi::new();
    match api.market(symbol) {
        Ok(json) => {
            let value = crate::core::output::parse_json_or_text(&json);
            println!(
                "{}",
                crate::core::output::explained_with_raw(
                    "market",
                    "종목 현재가와 기업 기본 정보 조회 결과",
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
                            "symbol",
                            "조회한 종목코드",
                            value
                                .get("symbol")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null),
                        ),
                        crate::core::output::field(
                            "price",
                            "현재가와 시세 관련 값",
                            value
                                .get("price")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!({})),
                        ),
                        crate::core::output::field(
                            "company",
                            "종목 및 기업 기본 정보",
                            value
                                .get("company")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!({})),
                        ),
                    ],
                    serde_json::Value::Null,
                )
            );
        }
        Err(err) => eprintln!(
            "{}",
            crate::core::output::error("market", "종목 정보 조회 실패", &err)
        ),
    }
}

fn handle_market_history(command: &MarketHistoryCommand) {
    let api = KisApi::new();
    let adjusted = !command.raw_price;
    match api.market_history(
        &command.symbol,
        &command.from,
        &command.to,
        &command.period,
        adjusted,
    ) {
        Ok(json) => {
            let value = crate::core::output::parse_json_or_text(&json);
            println!(
                "{}",
                crate::core::output::explained_with_raw(
                    "market history",
                    "종목 기간별 OHLCV 가격 히스토리 조회 결과",
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
                            "symbol",
                            "조회한 종목코드",
                            value
                                .get("symbol")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null),
                        ),
                        crate::core::output::field(
                            "period",
                            "가격 봉 단위. D=일봉, W=주봉, M=월봉, Y=년봉",
                            value
                                .get("period")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null),
                        ),
                        crate::core::output::field(
                            "date_range",
                            "조회 요청 기간",
                            serde_json::json!({
                                "from": value.get("start_date").cloned().unwrap_or(serde_json::Value::Null),
                                "to": value.get("end_date").cloned().unwrap_or(serde_json::Value::Null),
                            }),
                        ),
                        crate::core::output::field(
                            "adjusted",
                            "수정주가 조회 여부. false이면 원주가 조회",
                            value
                                .get("adjusted")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null),
                        ),
                        crate::core::output::field(
                            "count",
                            "조회된 가격 봉 개수",
                            value
                                .get("count")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null),
                        ),
                        crate::core::output::field(
                            "candles",
                            "날짜 오름차순 OHLCV 배열. open/high/low/close/volume/trading_value는 문자열 숫자로 보존한다.",
                            value
                                .get("candles")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!([])),
                        ),
                        crate::core::output::field(
                            "summary",
                            "KIS가 반환한 종목 요약 정보",
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
        Err(err) => eprintln!(
            "{}",
            crate::core::output::error("market history", "기간별 시세 조회 실패", &err)
        ),
    }
}
