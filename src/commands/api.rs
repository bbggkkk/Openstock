use crate::apis::create_default_registry;
use crate::apis::kis::{login::KisLoginArguments, KisApi};
use crate::core::TraderApi;
use clap::{Args, Subcommand};
use std::path::Path;

#[derive(Subcommand)]
pub enum ApiCommands {
    /// 증권사 API 목록 조회
    List,

    /// 한국투자증권(KIS) 실전 API 로그인 (접근토큰 발급)
    Login(KisLoginCommand),

    /// 한국투자증권(KIS) API 직접 호출
    Call(KisCallCommand),
}

#[derive(Args)]
pub struct KisLoginCommand {
    /// 한국투자증권 Open API 앱키 (.env의 KIS_APPKEY보다 우선)
    #[arg(long)]
    appkey: Option<String>,

    /// 한국투자증권 Open API 앱시크릿 (.env의 KIS_APPSECRET보다 우선)
    #[arg(long)]
    appsecret: Option<String>,

    /// 유효한 기존 접근토큰이 있어도 새로 발급
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
pub struct KisCallCommand {
    /// 호출할 KIS API 경로 또는 전체 URL
    endpoint: String,

    /// 요청 파라미터. KEY=VALUE 형식이며 tr_id는 요청 헤더로 전송
    #[arg(long = "param", short = 'p')]
    params: Vec<String>,
}

pub fn handle_api(sub: &ApiCommands) {
    match sub {
        ApiCommands::List => {
            let registry = create_default_registry();
            let apis = registry
                .list()
                .iter()
                .map(|api| api_catalog(api.as_ref()))
                .collect::<Vec<_>>();
            println!(
                "{}",
                crate::core::output::explained(
                    "api list",
                    "등록된 증권사 API 목록. AI 에이전트가 사용할 수 있도록 각 API의 목적, 인증 요구사항, 지원 명령, 입력 계약, 출력 계약, 부작용 여부를 포함한다.",
                    vec![
                        crate::core::output::field(
                            "count",
                            "등록된 증권사 API 개수",
                            serde_json::json!(apis.len()),
                        ),
                        crate::core::output::field(
                            "apis",
                            "사용 가능한 증권사 API 목록. 각 항목의 capabilities는 CLI 명령과 대응되며 side_effect가 none이면 조회, financial_order이면 실전 주문 전송이다.",
                            serde_json::json!(apis),
                        ),
                    ],
                )
            );
        }
        ApiCommands::Login(command) => {
            let env = crate::core::dotenv::read_env(Path::new(".env"));

            let appkey = command
                .appkey
                .clone()
                .or_else(|| env.get("KIS_APPKEY").cloned())
                .unwrap_or_default();
            let appsecret = command
                .appsecret
                .clone()
                .or_else(|| env.get("KIS_APPSECRET").cloned())
                .unwrap_or_default();

            let args = KisLoginArguments::new(appkey, appsecret, command.force);
            let mut api = KisApi::new();

            match api.login(&args) {
                Ok(()) => println!(
                    "{}",
                    crate::core::output::explained(
                        "api login",
                        "한국투자증권 실전 API 접근토큰 발급 또는 기존 토큰 재사용 결과. 이 명령은 인증 상태를 준비하며 주식 주문이나 조회 요청을 직접 실행하지 않는다.",
                        vec![
                            crate::core::output::field(
                                "broker",
                                "로그인 대상 증권사 API",
                                serde_json::json!(api.id()),
                            ),
                            crate::core::output::field(
                                "status",
                                "로그인 처리 결과",
                                serde_json::json!("success"),
                            ),
                            crate::core::output::field(
                                "force",
                                "기존 유효 토큰을 무시하고 새로 발급했는지 여부",
                                serde_json::json!(command.force),
                            ),
                            crate::core::output::field(
                                "credential_source",
                                "appkey/appsecret 입력 출처. CLI 옵션이 있으면 우선 사용하고 없으면 .env의 KIS_APPKEY/KIS_APPSECRET을 사용한다.",
                                serde_json::json!({
                                    "appkey": if command.appkey.is_some() { "cli_argument" } else { ".env:KIS_APPKEY" },
                                    "appsecret": if command.appsecret.is_some() { "cli_argument" } else { ".env:KIS_APPSECRET" },
                                }),
                            ),
                            crate::core::output::field(
                                "token_storage",
                                "발급 또는 재사용된 접근토큰이 저장되는 위치와 키",
                                serde_json::json!({
                                    "file": ".env",
                                    "access_token_key": "KIS_ACCESS_TOKEN",
                                    "expiration_key": "KIS_ACCESS_TOKEN_EXPIRED_AT",
                                }),
                            ),
                            crate::core::output::field(
                                "side_effect",
                                "명령의 외부 부작용. 인증 토큰과 인증 정보를 .env에 저장할 수 있지만 금융 주문은 발생하지 않는다.",
                                serde_json::json!("writes_auth_state"),
                            ),
                        ],
                    )
                ),
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error("api login", "KIS 로그인 실패", &err)
                ),
            }
        }
        ApiCommands::Call(command) => {
            let params = match parse_params(&command.params) {
                Ok(params) => params,
                Err(err) => {
                    eprintln!(
                        "{}",
                        crate::core::output::error("api call", "KIS API 호출 실패", &err)
                    );
                    return;
                }
            };
            let param_refs = params
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect::<Vec<_>>();
            let api = KisApi::new();

            match api.call(&command.endpoint, &param_refs) {
                Ok(json) => {
                    let value = crate::core::output::parse_json_or_text(&json);
                    println!(
                        "{}",
                        crate::core::output::explained_with_raw(
                        "api call",
                            "지정한 KIS API 엔드포인트 직접 호출 결과. tr_id 파라미터는 KIS 거래 ID 헤더로 이동하고 나머지 파라미터는 query string으로 전송한다.",
                            vec![
                                crate::core::output::field(
                                    "broker",
                                    "호출에 사용한 증권사 API",
                                    serde_json::json!(api.id()),
                                ),
                                crate::core::output::field(
                                    "endpoint",
                                    "호출한 API 경로 또는 URL",
                                    serde_json::json!(command.endpoint),
                                ),
                                crate::core::output::field(
                                    "params",
                                    "요청에 사용한 파라미터 목록. tr_id는 HTTP 헤더, 나머지는 URL query parameter로 해석한다.",
                                    serde_json::json!(params_for_output(&params)),
                                ),
                                crate::core::output::field(
                                    "request_semantics",
                                    "KIS 직접 호출 요청 해석 정보",
                                    serde_json::json!({
                                        "http_method": "GET",
                                        "auth": "KIS_ACCESS_TOKEN Bearer token with KIS_APPKEY and KIS_APPSECRET headers",
                                        "tr_id_handling": "param named tr_id is sent as request header tr_id",
                                        "side_effect": "unknown; depends on endpoint. Use typed commands for known read/order actions.",
                                    }),
                                ),
                                crate::core::output::field(
                                    "response",
                                    "API 응답 값",
                                    value.clone(),
                                ),
                                crate::core::output::field(
                                    "response_semantics",
                                    "응답의 주요 top-level 필드 의미. KIS rt_cd가 0이면 성공, msg1은 사람이 읽는 응답 메시지, output/output1/output2는 API별 본문 데이터다.",
                                    explain_response_keys(&value),
                                ),
                            ],
                            value,
                        )
                    );
                }
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error("api call", "KIS API 호출 실패", &err)
                ),
            }
        }
    }
}

fn api_catalog(api: &dyn TraderApi) -> serde_json::Value {
    serde_json::json!({
        "id": api.id(),
        "name": api.name(),
        "description": api.description(),
        "ai_description": "KIS is the configured live Korea Investment & Securities broker API. It supports authentication, direct endpoint calls, account status lookup, market data lookup, live domestic stock buy/sell orders, and order status lookup.",
        "info": api.info()
            .into_iter()
            .map(|(key, value)| serde_json::json!({
                "name": key,
                "description": "Broker metadata key-value pair",
                "value": value,
            }))
            .collect::<Vec<_>>(),
        "credential_requirements": [
            {
                "name": "KIS_APPKEY",
                "description": "KIS Open API application key used for authentication and every broker API request.",
                "required_for": ["api login", "api call", "account status", "market", "order buy", "order sell", "order status"],
                "source": ".env or api login --appkey"
            },
            {
                "name": "KIS_APPSECRET",
                "description": "KIS Open API application secret used for authentication and every broker API request.",
                "required_for": ["api login", "api call", "account status", "market", "order buy", "order sell", "order status"],
                "source": ".env or api login --appsecret"
            },
            {
                "name": "KIS_ACCESS_TOKEN",
                "description": "Bearer access token issued by api login. Read commands and order commands require it.",
                "required_for": ["api call", "account status", "market", "order buy", "order sell", "order status"],
                "source": ".env written by api login"
            },
            {
                "name": "KIS_ACCOUNT",
                "description": "Live account identifier in CANO-ACNT_PRDT_CD format, for example 12345678-01. Required for account and order commands.",
                "required_for": ["account status", "order buy", "order sell", "order status"],
                "source": ".env"
            }
        ],
        "capabilities": [
            {
                "command": "api login",
                "purpose": "Prepare authentication by issuing or reusing a live KIS access token.",
                "inputs": [
                    {"name": "--appkey", "required": false, "description": "Overrides .env KIS_APPKEY."},
                    {"name": "--appsecret", "required": false, "description": "Overrides .env KIS_APPSECRET."},
                    {"name": "--force", "required": false, "description": "Issue a new token even if a valid token already exists."}
                ],
                "output_contract": "Explained JSON with broker, status, force, credential_source, token_storage, and side_effect fields.",
                "side_effect": "writes_auth_state"
            },
            {
                "command": "api call <endpoint> --param KEY=VALUE",
                "purpose": "Call an arbitrary KIS endpoint using the configured access token.",
                "inputs": [
                    {"name": "endpoint", "required": true, "description": "KIS API path or full URL."},
                    {"name": "--param", "required": false, "description": "KEY=VALUE request parameter. tr_id is sent as an HTTP header."}
                ],
                "output_contract": "Explained JSON with request metadata, parsed response, and response_semantics.",
                "side_effect": "unknown"
            },
            {
                "command": "account status",
                "purpose": "Read account cash balance, total evaluation, profit/loss, and holdings.",
                "inputs": [],
                "output_contract": "Explained JSON with broker, account, balance, holdings, and raw KIS response.",
                "side_effect": "none"
            },
            {
                "command": "market <symbol>",
                "purpose": "Read current price and company/basic stock information for a domestic stock code.",
                "inputs": [{"name": "symbol", "required": true, "description": "Six digit domestic stock code such as 005930."}],
                "output_contract": "Explained JSON with broker, symbol, price, company, and raw KIS response.",
                "side_effect": "none"
            },
            {
                "command": "order buy <symbol> --qty <qty> (--price <price>|--market)",
                "purpose": "Submit a live domestic stock buy order.",
                "inputs": [
                    {"name": "symbol", "required": true, "description": "Six digit domestic stock code."},
                    {"name": "--qty", "required": true, "description": "Order quantity."},
                    {"name": "--price", "required": false, "description": "Limit order price. Required unless --market is used."},
                    {"name": "--market", "required": false, "description": "Submit market order."}
                ],
                "output_contract": "Explained JSON with broker, side, symbol, qty, order_type, price, order, and raw KIS response.",
                "side_effect": "financial_order"
            },
            {
                "command": "order sell <symbol> --qty <qty> (--price <price>|--market)",
                "purpose": "Submit a live domestic stock sell order.",
                "inputs": [
                    {"name": "symbol", "required": true, "description": "Six digit domestic stock code."},
                    {"name": "--qty", "required": true, "description": "Order quantity."},
                    {"name": "--price", "required": false, "description": "Limit order price. Required unless --market is used."},
                    {"name": "--market", "required": false, "description": "Submit market order."}
                ],
                "output_contract": "Explained JSON with broker, side, symbol, qty, order_type, price, order, and raw KIS response.",
                "side_effect": "financial_order"
            },
            {
                "command": "order status [order_no] --from YYYYMMDD --to YYYYMMDD",
                "purpose": "Read order and execution status for the configured account.",
                "inputs": [
                    {"name": "order_no", "required": false, "description": "Specific order number to filter."},
                    {"name": "--from", "required": false, "description": "Start date in YYYYMMDD. Defaults to today in KST."},
                    {"name": "--to", "required": false, "description": "End date in YYYYMMDD. Defaults to today in KST."}
                ],
                "output_contract": "Explained JSON with broker, account, order_no, orders, summary, and raw KIS response.",
                "side_effect": "none"
            }
        ]
    })
}

fn params_for_output(params: &[(String, String)]) -> Vec<serde_json::Value> {
    params
        .iter()
        .map(|(key, value)| {
            serde_json::json!({
                "name": key,
                "value": value,
                "transport": if key.eq_ignore_ascii_case("tr_id") { "http_header" } else { "query_parameter" },
                "description": if key.eq_ignore_ascii_case("tr_id") {
                    "KIS transaction id. Sent as the tr_id HTTP header."
                } else {
                    "Endpoint-specific request parameter. Sent as URL query parameter."
                },
            })
        })
        .collect()
}

fn explain_response_keys(value: &serde_json::Value) -> serde_json::Value {
    let Some(object) = value.as_object() else {
        return serde_json::json!([{
            "name": "response",
            "description": "Response is not a JSON object.",
            "value_type": value_type(value),
        }]);
    };

    serde_json::json!(object
        .iter()
        .map(|(key, value)| {
            serde_json::json!({
                "name": key,
                "description": response_key_description(key),
                "value_type": value_type(value),
            })
        })
        .collect::<Vec<_>>())
}

fn response_key_description(key: &str) -> &'static str {
    match key {
        "rt_cd" => {
            "KIS result code. 0 usually means success; non-zero indicates API-level failure."
        }
        "msg_cd" => "KIS message code for the response.",
        "msg1" => "Human-readable KIS response message.",
        "output" => "Main response payload for APIs that return a single output object.",
        "output1" => "First response payload. Meaning depends on endpoint, often list/detail rows.",
        "output2" => "Second response payload. Meaning depends on endpoint, often summary values.",
        "ctx_area_fk100" => "Pagination cursor field used by some KIS list APIs.",
        "ctx_area_nk100" => "Pagination cursor field used by some KIS list APIs.",
        _ => "Endpoint-specific response field returned by KIS.",
    }
}

fn value_type(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn parse_params(params: &[String]) -> Result<Vec<(String, String)>, String> {
    params
        .iter()
        .map(|param| {
            let (key, value) = param
                .split_once('=')
                .ok_or_else(|| format!("파라미터는 KEY=VALUE 형식이어야 합니다: {}", param))?;
            if key.trim().is_empty() {
                return Err(format!("파라미터 키가 비어 있습니다: {}", param));
            }
            Ok((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}
