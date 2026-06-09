use crate::core::stock::{Stock, StockKind, StockMarket};
use crate::providers::kind;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum UniverseCommands {
    /// KIND 상장법인목록을 내려받아 로컬 universe 캐시를 갱신
    Sync(UniverseSyncCommand),

    /// 로컬 universe 캐시 상태 조회
    Status,

    /// 로컬 universe 종목 목록 조회
    List(UniverseListCommand),

    /// 로컬 universe를 chunk 단위로 분할
    Chunks(UniverseChunksCommand),

    /// 로컬 universe 캐시의 크기와 대표 값을 검증
    Validate(UniverseValidateCommand),
}

#[derive(Args)]
pub struct UniverseSyncCommand {
    /// 오늘 이미 갱신된 캐시가 있어도 다시 다운로드
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct UniverseListCommand {
    /// 시장 필터: KOSPI, KOSDAQ, KONEX
    #[arg(long)]
    pub market: Option<String>,

    /// 종목 유형 필터: common_stock, preferred_stock, etf, etn, reit, spac
    #[arg(long)]
    pub kind: Option<String>,

    /// 반환할 최대 종목 수
    #[arg(long, default_value_t = 100)]
    pub limit: usize,

    /// 건너뛸 종목 수
    #[arg(long, default_value_t = 0)]
    pub offset: usize,
}

#[derive(Args)]
pub struct UniverseChunksCommand {
    /// 시장 필터: KOSPI, KOSDAQ, KONEX
    #[arg(long)]
    pub market: Option<String>,

    /// 종목 유형 필터: common_stock, preferred_stock, etf, etn, reit, spac
    #[arg(long)]
    pub kind: Option<String>,

    /// chunk당 종목 수
    #[arg(long, default_value_t = 100)]
    pub size: usize,
}

#[derive(Args)]
pub struct UniverseValidateCommand {
    /// 기대하는 최소 전체 종목 수
    #[arg(long, default_value_t = 2500)]
    pub min_count: usize,

    /// 기대하는 최소 KOSPI 종목 수
    #[arg(long, default_value_t = 700)]
    pub min_kospi: usize,

    /// 기대하는 최소 KOSDAQ 종목 수
    #[arg(long, default_value_t = 1500)]
    pub min_kosdaq: usize,

    /// 기대하는 최소 KONEX 종목 수
    #[arg(long, default_value_t = 50)]
    pub min_konex: usize,

    /// 반드시 존재해야 하는 종목. SYMBOL=NAME 형식
    #[arg(long = "expect")]
    pub expects: Vec<String>,
}

pub fn handle_universe(sub: &UniverseCommands) {
    match sub {
        UniverseCommands::Sync(command) => match kind::universe::load_or_refresh(command.force) {
            Ok(snapshot) => {
                print_snapshot("universe sync", "종목 universe 캐시 갱신 결과", &snapshot)
            }
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("universe sync", "종목 universe 캐시 갱신 실패", &err)
            ),
        },
        UniverseCommands::Status => match kind::universe::load_cached() {
            Ok(Some(snapshot)) => {
                print_snapshot("universe status", "종목 universe 캐시 상태", &snapshot)
            }
            Ok(None) => eprintln!(
                "{}",
                crate::core::output::error(
                    "universe status",
                    "종목 universe 캐시 상태 조회 실패",
                    "로컬 universe 캐시가 없습니다. `openstock universe sync`를 먼저 실행하세요.",
                )
            ),
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error(
                    "universe status",
                    "종목 universe 캐시 상태 조회 실패",
                    &err
                )
            ),
        },
        UniverseCommands::List(command) => match kind::universe::load_or_refresh(false) {
            Ok(snapshot) => match filter_stocks(
                &snapshot.stocks,
                command.market.as_deref(),
                command.kind.as_deref(),
            ) {
                Ok(filtered) => {
                    let page = filtered
                        .iter()
                        .skip(command.offset)
                        .take(command.limit)
                        .cloned()
                        .collect::<Vec<_>>();
                    println!(
                        "{}",
                        crate::core::output::explained_with_raw(
                            "universe list",
                            "로컬 universe 캐시에서 종목 목록을 조회한 결과",
                            vec![
                                crate::core::output::field(
                                    "source",
                                    "universe 원천 데이터",
                                    serde_json::json!(snapshot.meta.source),
                                ),
                                crate::core::output::field(
                                    "cache_date",
                                    "캐시 기준일",
                                    serde_json::json!(snapshot.meta.date),
                                ),
                                crate::core::output::field(
                                    "total_count",
                                    "필터 적용 전 전체 종목 수",
                                    serde_json::json!(snapshot.stocks.len()),
                                ),
                                crate::core::output::field(
                                    "filtered_count",
                                    "필터 적용 후 종목 수",
                                    serde_json::json!(filtered.len()),
                                ),
                                crate::core::output::field(
                                    "offset",
                                    "조회 시작 offset",
                                    serde_json::json!(command.offset),
                                ),
                                crate::core::output::field(
                                    "limit",
                                    "최대 반환 종목 수",
                                    serde_json::json!(command.limit),
                                ),
                                crate::core::output::field(
                                    "stocks",
                                    "조회된 종목 목록",
                                    serde_json::json!(page),
                                ),
                            ],
                            serde_json::json!({
                                "meta": snapshot.meta,
                                "stocks": page,
                            }),
                        )
                    );
                }
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error(
                        "universe list",
                        "종목 universe 목록 조회 실패",
                        &err
                    )
                ),
            },
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("universe list", "종목 universe 목록 조회 실패", &err)
            ),
        },
        UniverseCommands::Chunks(command) => match kind::universe::load_or_refresh(false) {
            Ok(snapshot) => match filter_stocks(
                &snapshot.stocks,
                command.market.as_deref(),
                command.kind.as_deref(),
            ) {
                Ok(filtered) => {
                    let chunks = kind::universe::chunk_stocks(&filtered, command.size);
                    println!(
                        "{}",
                        crate::core::output::explained_with_raw(
                            "universe chunks",
                            "로컬 universe 캐시를 scan 가능한 chunk로 분할한 결과",
                            vec![
                                crate::core::output::field(
                                    "source",
                                    "universe 원천 데이터",
                                    serde_json::json!(snapshot.meta.source),
                                ),
                                crate::core::output::field(
                                    "cache_date",
                                    "캐시 기준일",
                                    serde_json::json!(snapshot.meta.date),
                                ),
                                crate::core::output::field(
                                    "filtered_count",
                                    "chunk 생성 대상 종목 수",
                                    serde_json::json!(filtered.len()),
                                ),
                                crate::core::output::field(
                                    "chunk_size",
                                    "chunk당 최대 종목 수",
                                    serde_json::json!(command.size.max(1)),
                                ),
                                crate::core::output::field(
                                    "chunk_count",
                                    "생성된 chunk 수",
                                    serde_json::json!(chunks.len()),
                                ),
                                crate::core::output::field(
                                    "chunks",
                                    "생성된 chunk 목록. start_symbol과 end_symbol은 문자열 정렬 기준 cursor 경계다.",
                                    serde_json::json!(chunks),
                                ),
                            ],
                            serde_json::json!({
                                "meta": snapshot.meta,
                                "chunks": chunks,
                            }),
                        )
                    );
                }
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error(
                        "universe chunks",
                        "종목 universe chunk 생성 실패",
                        &err
                    )
                ),
            },
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error(
                    "universe chunks",
                    "종목 universe chunk 생성 실패",
                    &err
                )
            ),
        },
        UniverseCommands::Validate(command) => match kind::universe::load_or_refresh(false) {
            Ok(snapshot) => match validate_universe(&snapshot.stocks, command) {
                Ok(report) => println!(
                    "{}",
                    crate::core::output::explained_with_raw(
                        "universe validate",
                        "로컬 universe 캐시의 크기, 중복, 필수 필드, 대표 종목 값을 검증한 결과",
                        vec![
                            crate::core::output::field(
                                "valid",
                                "검증 통과 여부",
                                serde_json::json!(report.valid),
                            ),
                            crate::core::output::field(
                                "stock_count",
                                "검증 대상 전체 종목 수",
                                serde_json::json!(report.stock_count),
                            ),
                            crate::core::output::field(
                                "counts_by_market",
                                "시장별 종목 수",
                                serde_json::json!(report.counts_by_market),
                            ),
                            crate::core::output::field(
                                "checks",
                                "개별 검증 항목과 결과",
                                serde_json::json!(report.checks),
                            ),
                            crate::core::output::field(
                                "errors",
                                "실패한 검증 항목 설명",
                                serde_json::json!(report.errors),
                            ),
                        ],
                        serde_json::json!({
                            "meta": snapshot.meta,
                            "report": report,
                        }),
                    )
                ),
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error(
                        "universe validate",
                        "종목 universe 검증 실패",
                        &err
                    )
                ),
            },
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("universe validate", "종목 universe 검증 실패", &err)
            ),
        },
    }
}

fn print_snapshot(command: &str, description: &str, snapshot: &kind::universe::UniverseSnapshot) {
    println!(
        "{}",
        crate::core::output::explained_with_raw(
            command,
            description,
            vec![
                crate::core::output::field(
                    "source",
                    "universe 원천 데이터",
                    serde_json::json!(snapshot.meta.source),
                ),
                crate::core::output::field(
                    "source_url",
                    "universe를 내려받은 원천 URL",
                    serde_json::json!(snapshot.meta.source_url),
                ),
                crate::core::output::field(
                    "cache_date",
                    "캐시 기준일",
                    serde_json::json!(snapshot.meta.date),
                ),
                crate::core::output::field(
                    "refreshed_at",
                    "캐시 갱신 시각",
                    serde_json::json!(snapshot.meta.refreshed_at),
                ),
                crate::core::output::field(
                    "refreshed",
                    "이번 명령에서 새로 다운로드했는지 여부",
                    serde_json::json!(snapshot.refreshed),
                ),
                crate::core::output::field(
                    "stock_count",
                    "캐시에 저장된 전체 종목 수",
                    serde_json::json!(snapshot.meta.stock_count),
                ),
                crate::core::output::field(
                    "counts_by_market",
                    "시장별 종목 수",
                    serde_json::json!(snapshot.meta.counts_by_market),
                ),
            ],
            serde_json::json!(snapshot.meta),
        )
    );
}

fn filter_stocks(
    stocks: &[Stock],
    market: Option<&str>,
    kind: Option<&str>,
) -> Result<Vec<Stock>, String> {
    let market = market.map(parse_market).transpose()?;
    let kind = kind.map(parse_kind).transpose()?;
    let mut filtered = stocks
        .iter()
        .filter(|stock| market.map(|value| stock.market == value).unwrap_or(true))
        .filter(|stock| kind.map(|value| stock.kind == value).unwrap_or(true))
        .cloned()
        .collect::<Vec<_>>();
    filtered.sort_by(|a, b| {
        a.market
            .cmp(&b.market)
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.id.symbol.cmp(&b.id.symbol))
    });
    Ok(filtered)
}

fn parse_market(value: &str) -> Result<StockMarket, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "KOSPI" => Ok(StockMarket::Kospi),
        "KOSDAQ" => Ok(StockMarket::Kosdaq),
        "KONEX" => Ok(StockMarket::Konex),
        _ => Err(format!("지원하지 않는 시장 필터입니다: {}", value)),
    }
}

fn parse_kind(value: &str) -> Result<StockKind, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "common_stock" | "common" => Ok(StockKind::CommonStock),
        "preferred_stock" | "preferred" => Ok(StockKind::PreferredStock),
        "etf" => Ok(StockKind::Etf),
        "etn" => Ok(StockKind::Etn),
        "reit" => Ok(StockKind::Reit),
        "spac" => Ok(StockKind::Spac),
        _ => Err(format!("지원하지 않는 종목 유형 필터입니다: {}", value)),
    }
}

#[derive(serde::Serialize)]
struct UniverseValidationReport {
    valid: bool,
    stock_count: usize,
    counts_by_market: serde_json::Value,
    checks: Vec<ValidationCheck>,
    errors: Vec<String>,
}

#[derive(serde::Serialize)]
struct ValidationCheck {
    name: &'static str,
    description: &'static str,
    valid: bool,
    expected: serde_json::Value,
    actual: serde_json::Value,
}

fn validate_universe(
    stocks: &[Stock],
    command: &UniverseValidateCommand,
) -> Result<UniverseValidationReport, String> {
    let mut checks = Vec::new();
    let mut errors = Vec::new();
    let counts = market_counts(stocks);

    push_check(
        &mut checks,
        &mut errors,
        "min_count",
        "전체 종목 수가 기대 최소값 이상인지 검증",
        serde_json::json!({ "min": command.min_count }),
        serde_json::json!(stocks.len()),
        stocks.len() >= command.min_count,
    );
    push_check(
        &mut checks,
        &mut errors,
        "min_kospi",
        "KOSPI 종목 수가 기대 최소값 이상인지 검증",
        serde_json::json!({ "min": command.min_kospi }),
        serde_json::json!(counts.kospi),
        counts.kospi >= command.min_kospi,
    );
    push_check(
        &mut checks,
        &mut errors,
        "min_kosdaq",
        "KOSDAQ 종목 수가 기대 최소값 이상인지 검증",
        serde_json::json!({ "min": command.min_kosdaq }),
        serde_json::json!(counts.kosdaq),
        counts.kosdaq >= command.min_kosdaq,
    );
    push_check(
        &mut checks,
        &mut errors,
        "min_konex",
        "KONEX 종목 수가 기대 최소값 이상인지 검증",
        serde_json::json!({ "min": command.min_konex }),
        serde_json::json!(counts.konex),
        counts.konex >= command.min_konex,
    );

    let duplicate_symbols = duplicate_symbols(stocks);
    push_check(
        &mut checks,
        &mut errors,
        "unique_symbols",
        "종목코드가 중복 없이 유일한지 검증",
        serde_json::json!([]),
        serde_json::json!(duplicate_symbols),
        duplicate_symbols.is_empty(),
    );

    let invalid_required = stocks
        .iter()
        .filter(|stock| stock.id.symbol.trim().is_empty() || stock.name.trim().is_empty())
        .map(|stock| stock.id.symbol.clone())
        .collect::<Vec<_>>();
    push_check(
        &mut checks,
        &mut errors,
        "required_fields",
        "종목코드와 종목명이 비어 있지 않은지 검증",
        serde_json::json!([]),
        serde_json::json!(invalid_required),
        invalid_required.is_empty(),
    );

    let default_expects = if command.expects.is_empty() {
        vec![
            "005930=삼성전자".to_string(),
            "000020=동화약품".to_string(),
            "035720=카카오".to_string(),
        ]
    } else {
        command.expects.clone()
    };

    for expect in default_expects {
        let (symbol, name) = expect
            .split_once('=')
            .ok_or_else(|| format!("--expect 값은 SYMBOL=NAME 형식이어야 합니다: {}", expect))?;
        let actual = stocks.iter().find(|stock| stock.id.symbol == symbol);
        let valid = actual.map(|stock| stock.name == name).unwrap_or(false);
        push_check(
            &mut checks,
            &mut errors,
            "expected_symbol",
            "대표 종목코드가 기대 종목명으로 존재하는지 검증",
            serde_json::json!({ "symbol": symbol, "name": name }),
            serde_json::json!(actual.map(|stock| &stock.name)),
            valid,
        );
    }

    Ok(UniverseValidationReport {
        valid: errors.is_empty(),
        stock_count: stocks.len(),
        counts_by_market: serde_json::json!({
            "KOSPI": counts.kospi,
            "KOSDAQ": counts.kosdaq,
            "KONEX": counts.konex,
            "OTHER": counts.other,
            "UNKNOWN": counts.unknown,
        }),
        checks,
        errors,
    })
}

fn push_check(
    checks: &mut Vec<ValidationCheck>,
    errors: &mut Vec<String>,
    name: &'static str,
    description: &'static str,
    expected: serde_json::Value,
    actual: serde_json::Value,
    valid: bool,
) {
    if !valid {
        errors.push(format!("{} 검증 실패", name));
    }
    checks.push(ValidationCheck {
        name,
        description,
        valid,
        expected,
        actual,
    });
}

struct MarketCounts {
    kospi: usize,
    kosdaq: usize,
    konex: usize,
    other: usize,
    unknown: usize,
}

fn market_counts(stocks: &[Stock]) -> MarketCounts {
    let mut counts = MarketCounts {
        kospi: 0,
        kosdaq: 0,
        konex: 0,
        other: 0,
        unknown: 0,
    };

    for stock in stocks {
        match stock.market {
            StockMarket::Kospi => counts.kospi += 1,
            StockMarket::Kosdaq => counts.kosdaq += 1,
            StockMarket::Konex => counts.konex += 1,
            StockMarket::Other => counts.other += 1,
            StockMarket::Unknown => counts.unknown += 1,
        }
    }

    counts
}

fn duplicate_symbols(stocks: &[Stock]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut duplicates = std::collections::BTreeSet::new();
    for stock in stocks {
        if !seen.insert(stock.id.symbol.as_str()) {
            duplicates.insert(stock.id.symbol.clone());
        }
    }
    duplicates.into_iter().collect()
}
