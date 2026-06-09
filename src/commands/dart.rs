use crate::providers::opendart::{corp_codes, disclosures, document};
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum DartCommands {
    /// OpenDART 상장사 고유번호 매핑 캐시 갱신
    Sync(DartSyncCommand),

    /// OpenDART 고유번호 매핑 캐시 상태 조회
    Status,

    /// 종목코드로 OpenDART 고유번호 조회
    Corp(DartCorpCommand),

    /// 전체 또는 특정 종목의 공시목록 조회
    Disclosures(DartDisclosuresCommand),

    /// 접수번호로 공시서류 원본파일 조회
    Document(DartDocumentCommand),

    /// 종목코드로 공시목록과 선택한 공시 원문 조회
    Show(DartShowCommand),
}

#[derive(Args)]
pub struct DartSyncCommand {
    /// 오늘 이미 갱신된 캐시가 있어도 다시 다운로드
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct DartCorpCommand {
    /// KRX 종목코드
    pub symbol: String,
}

#[derive(Args)]
pub struct DartDisclosuresCommand {
    /// KRX 종목코드. 생략하면 전체 공시 검색
    pub symbol: Option<String>,

    /// DART 고유번호. symbol보다 우선
    #[arg(long)]
    pub corp_code: Option<String>,

    /// 검색 시작일 YYYYMMDD
    #[arg(long = "from")]
    pub from: Option<String>,

    /// 검색 종료일 YYYYMMDD
    #[arg(long = "to")]
    pub to: Option<String>,

    /// 법인구분: Y=유가, K=코스닥, N=코넥스, E=기타
    #[arg(long)]
    pub corp_cls: Option<String>,

    /// 페이지 번호
    #[arg(long, default_value_t = 1)]
    pub page_no: u32,

    /// 페이지당 건수 (1~100)
    #[arg(long, default_value_t = 20)]
    pub page_count: u32,
}

#[derive(Args)]
pub struct DartDocumentCommand {
    /// DART 공시 접수번호
    pub rcept_no: String,

    /// 캐시가 있어도 다시 다운로드
    #[arg(long)]
    pub force: bool,

    /// 출력할 본문 텍스트 최대 글자 수
    #[arg(long, default_value_t = 20_000)]
    pub max_chars: usize,
}

#[derive(Args)]
pub struct DartShowCommand {
    /// KRX 종목코드
    pub symbol: String,

    /// 검색 시작일 YYYYMMDD
    #[arg(long = "from")]
    pub from: Option<String>,

    /// 검색 종료일 YYYYMMDD
    #[arg(long = "to")]
    pub to: Option<String>,

    /// 공시목록에서 열람할 항목 번호. 1이면 최신 첫 번째 항목
    #[arg(long, default_value_t = 1)]
    pub index: usize,

    /// 공시목록 조회 건수 (1~100)
    #[arg(long, default_value_t = 20)]
    pub page_count: u32,

    /// 캐시가 있어도 원문을 다시 다운로드
    #[arg(long)]
    pub force: bool,

    /// 출력할 본문 텍스트 최대 글자 수
    #[arg(long, default_value_t = 20_000)]
    pub max_chars: usize,
}

pub fn handle_dart(sub: &DartCommands) {
    match sub {
        DartCommands::Sync(command) => match corp_codes::load_or_refresh(command.force) {
            Ok(snapshot) => print_corp_snapshot(
                "dart sync",
                "OpenDART 상장사 고유번호 매핑 캐시 갱신 결과",
                &snapshot,
            ),
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error(
                    "dart sync",
                    "OpenDART 고유번호 매핑 캐시 갱신 실패",
                    &err
                )
            ),
        },
        DartCommands::Status => match corp_codes::load_cached() {
            Ok(Some(snapshot)) => {
                print_corp_snapshot("dart status", "OpenDART 고유번호 매핑 캐시 상태", &snapshot)
            }
            Ok(None) => eprintln!(
                "{}",
                crate::core::output::error(
                    "dart status",
                    "OpenDART 고유번호 매핑 캐시 상태 조회 실패",
                    "로컬 OpenDART 캐시가 없습니다. `openstock dart sync`를 먼저 실행하세요.",
                )
            ),
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error(
                    "dart status",
                    "OpenDART 고유번호 매핑 캐시 상태 조회 실패",
                    &err
                )
            ),
        },
        DartCommands::Corp(command) => match corp_codes::load_or_refresh(false) {
            Ok(snapshot) => {
                let symbol = command.symbol.trim();
                match corp_codes::find_by_stock_code(&snapshot.corps, symbol) {
                    Some(corp) => println!(
                        "{}",
                        crate::core::output::explained_with_raw(
                            "dart corp",
                            "KRX 종목코드에 대응하는 OpenDART 고유번호 조회 결과",
                            vec![
                                crate::core::output::field(
                                    "stock_code",
                                    "KRX 종목코드",
                                    serde_json::json!(corp.stock_code),
                                ),
                                crate::core::output::field(
                                    "corp_code",
                                    "OpenDART 공시 API에서 사용하는 8자리 고유번호",
                                    serde_json::json!(corp.corp_code),
                                ),
                                crate::core::output::field(
                                    "corp_name",
                                    "OpenDART에 등록된 회사명",
                                    serde_json::json!(corp.corp_name),
                                ),
                                crate::core::output::field(
                                    "modify_date",
                                    "OpenDART 고유번호 정보의 최근 변경일자",
                                    serde_json::json!(corp.modify_date),
                                ),
                            ],
                            serde_json::json!({
                                "meta": snapshot.meta,
                                "corp": corp,
                            }),
                        )
                    ),
                    None => eprintln!(
                        "{}",
                        crate::core::output::error(
                            "dart corp",
                            "OpenDART 고유번호 조회 실패",
                            &format!(
                                "종목코드 {}에 대응하는 DART 고유번호를 찾지 못했습니다.",
                                symbol
                            ),
                        )
                    ),
                }
            }
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("dart corp", "OpenDART 고유번호 조회 실패", &err)
            ),
        },
        DartCommands::Disclosures(command) => match disclosure_query(command) {
            Ok((query, resolved)) => match disclosures::list(&query) {
                Ok(value) => print_disclosures(&query, &resolved, value),
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error(
                        "dart disclosures",
                        "OpenDART 공시목록 조회 실패",
                        &err
                    )
                ),
            },
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("dart disclosures", "OpenDART 공시목록 조회 실패", &err)
            ),
        },
        DartCommands::Document(command) => {
            let options = document::DartDocumentOptions {
                rcept_no: command.rcept_no.trim().to_string(),
                force: command.force,
                max_chars: command.max_chars,
            };
            match document::get(&options) {
                Ok(value) => print_document(value),
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error(
                        "dart document",
                        "OpenDART 공시서류 원본파일 조회 실패",
                        &err
                    )
                ),
            }
        }
        DartCommands::Show(command) => match show_query(command) {
            Ok((query, resolved)) => match disclosures::list(&query) {
                Ok(disclosures_value) => {
                    match selected_disclosure(&disclosures_value, command.index) {
                        Ok(selected) => {
                            let Some(rcept_no) =
                                selected.get("rcept_no").and_then(|value| value.as_str())
                            else {
                                eprintln!(
                                    "{}",
                                    crate::core::output::error(
                                        "dart show",
                                        "종목 공시 원문 조회 실패",
                                        "선택한 공시 항목에 rcept_no가 없습니다.",
                                    )
                                );
                                return;
                            };
                            let options = document::DartDocumentOptions {
                                rcept_no: rcept_no.to_string(),
                                force: command.force,
                                max_chars: command.max_chars,
                            };
                            match document::get(&options) {
                                Ok(document) => print_show(
                                    command,
                                    &query,
                                    &resolved,
                                    &disclosures_value,
                                    selected,
                                    document,
                                ),
                                Err(err) => eprintln!(
                                    "{}",
                                    crate::core::output::error(
                                        "dart show",
                                        "종목 공시 원문 조회 실패",
                                        &err
                                    )
                                ),
                            }
                        }
                        Err(err) => eprintln!(
                            "{}",
                            crate::core::output::error(
                                "dart show",
                                "종목 공시 원문 조회 실패",
                                &err
                            )
                        ),
                    }
                }
                Err(err) => eprintln!(
                    "{}",
                    crate::core::output::error("dart show", "종목 공시목록 조회 실패", &err)
                ),
            },
            Err(err) => eprintln!(
                "{}",
                crate::core::output::error("dart show", "종목 공시 원문 조회 실패", &err)
            ),
        },
    }
}

fn print_corp_snapshot(
    command: &str,
    description: &str,
    snapshot: &corp_codes::DartCorpCodeSnapshot,
) {
    println!(
        "{}",
        crate::core::output::explained_with_raw(
            command,
            description,
            vec![
                crate::core::output::field(
                    "source",
                    "공시정보 원천 데이터",
                    serde_json::json!(snapshot.meta.source),
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
                    "total_count",
                    "OpenDART 고유번호 전체 항목 수",
                    serde_json::json!(snapshot.meta.total_count),
                ),
                crate::core::output::field(
                    "listed_count",
                    "stock_code가 존재하는 상장사 매핑 수",
                    serde_json::json!(snapshot.meta.listed_count),
                ),
            ],
            serde_json::Value::Null,
        )
    );
}

fn disclosure_query(
    command: &DartDisclosuresCommand,
) -> Result<(disclosures::DisclosureQuery, serde_json::Value), String> {
    let mut resolved = serde_json::json!({
        "input_symbol": command.symbol,
        "input_corp_code": command.corp_code,
    });
    let corp_code = if let Some(corp_code) = command
        .corp_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(corp_code.to_string())
    } else if let Some(symbol) = command
        .symbol
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let snapshot = corp_codes::load_or_refresh(false)?;
        let corp = corp_codes::find_by_stock_code(&snapshot.corps, symbol).ok_or_else(|| {
            format!(
                "종목코드 {}에 대응하는 DART 고유번호를 찾지 못했습니다.",
                symbol
            )
        })?;
        resolved["resolved_corp"] = serde_json::json!(corp);
        Some(corp.corp_code.clone())
    } else {
        None
    };

    Ok((
        disclosures::DisclosureQuery {
            corp_code,
            bgn_de: command.from.clone(),
            end_de: command.to.clone(),
            corp_cls: command.corp_cls.clone(),
            page_no: command.page_no,
            page_count: command.page_count,
        },
        resolved,
    ))
}

fn show_query(
    command: &DartShowCommand,
) -> Result<(disclosures::DisclosureQuery, serde_json::Value), String> {
    if command.index == 0 {
        return Err("--index는 1 이상이어야 합니다.".to_string());
    }
    let symbol = command.symbol.trim();
    let snapshot = corp_codes::load_or_refresh(false)?;
    let corp = corp_codes::find_by_stock_code(&snapshot.corps, symbol).ok_or_else(|| {
        format!(
            "종목코드 {}에 대응하는 DART 고유번호를 찾지 못했습니다.",
            symbol
        )
    })?;

    Ok((
        disclosures::DisclosureQuery {
            corp_code: Some(corp.corp_code.clone()),
            bgn_de: command.from.clone(),
            end_de: command.to.clone(),
            corp_cls: None,
            page_no: 1,
            page_count: command.page_count,
        },
        serde_json::json!({
            "input_symbol": symbol,
            "resolved_corp": corp,
        }),
    ))
}

fn selected_disclosure(
    disclosures_value: &serde_json::Value,
    index: usize,
) -> Result<&serde_json::Value, String> {
    let list = disclosures_value
        .get("list")
        .and_then(|value| value.as_array())
        .ok_or("공시목록 응답에 list가 없습니다.")?;
    if list.is_empty() {
        return Err("조회된 공시가 없습니다.".to_string());
    }
    list.get(index - 1).ok_or_else(|| {
        format!(
            "--index {}에 해당하는 공시가 없습니다. 현재 조회된 공시는 {}건입니다.",
            index,
            list.len()
        )
    })
}

fn print_disclosures(
    query: &disclosures::DisclosureQuery,
    resolved: &serde_json::Value,
    value: serde_json::Value,
) {
    let list = value
        .get("list")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    println!(
        "{}",
        crate::core::output::explained_with_raw(
            "dart disclosures",
            "OpenDART 공시목록 조회 결과",
            vec![
                crate::core::output::field(
                    "corp_code",
                    "조회에 사용한 OpenDART 고유번호. 없으면 전체 공시 조회",
                    query
                        .corp_code
                        .as_ref()
                        .map(|value| serde_json::json!(value))
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "from",
                    "검색 시작일 YYYYMMDD",
                    query
                        .bgn_de
                        .as_ref()
                        .map(|value| serde_json::json!(value))
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "to",
                    "검색 종료일 YYYYMMDD",
                    query
                        .end_de
                        .as_ref()
                        .map(|value| serde_json::json!(value))
                        .unwrap_or(serde_json::Value::Null),
                ),
                crate::core::output::field(
                    "page",
                    "조회 페이지 정보",
                    serde_json::json!({
                        "page_no": query.page_no,
                        "page_count": query.page_count,
                        "total_count": value.get("total_count").cloned().unwrap_or(serde_json::Value::Null),
                        "total_page": value.get("total_page").cloned().unwrap_or(serde_json::Value::Null),
                    }),
                ),
                crate::core::output::field(
                    "resolved",
                    "종목코드를 입력한 경우 OpenDART 고유번호로 변환한 정보",
                    resolved.clone(),
                ),
                crate::core::output::field("disclosures", "조회된 공시 목록", list),
            ],
            serde_json::Value::Null,
        )
    );
}

fn print_document(value: document::DartDocument) {
    println!(
        "{}",
        crate::core::output::explained_with_raw(
            "dart document",
            "OpenDART 공시서류 원본파일 조회 및 텍스트 추출 결과",
            vec![
                crate::core::output::field(
                    "rcept_no",
                    "DART 공시 접수번호",
                    serde_json::json!(value.rcept_no),
                ),
                crate::core::output::field(
                    "source",
                    "공시서류 원본파일 제공 원천",
                    serde_json::json!(value.source),
                ),
                crate::core::output::field(
                    "viewer_url",
                    "브라우저에서 확인 가능한 DART 공시 뷰어 URL",
                    serde_json::json!(value.viewer_url),
                ),
                crate::core::output::field(
                    "cached",
                    "로컬 ZIP 캐시를 재사용했는지 여부",
                    serde_json::json!(value.cached),
                ),
                crate::core::output::field(
                    "zip_path",
                    "로컬에 저장된 공시서류 ZIP 캐시 경로",
                    serde_json::json!(value.zip_path),
                ),
                crate::core::output::field(
                    "zip_bytes",
                    "공시서류 ZIP 파일 크기(bytes)",
                    serde_json::json!(value.zip_bytes),
                ),
                crate::core::output::field(
                    "files",
                    "ZIP 내부 파일 목록과 각 파일에서 추출한 텍스트 길이",
                    serde_json::json!(value.files),
                ),
                crate::core::output::field(
                    "text",
                    "XML 태그를 제거한 공시 본문 텍스트. max_chars를 넘으면 앞부분만 제공",
                    serde_json::json!(value.text),
                ),
                crate::core::output::field(
                    "text_chars",
                    "출력된 본문 텍스트 글자 수",
                    serde_json::json!(value.text_chars),
                ),
                crate::core::output::field(
                    "truncated",
                    "본문이 max_chars 제한으로 잘렸는지 여부",
                    serde_json::json!(value.truncated),
                ),
            ],
            serde_json::Value::Null,
        )
    );
}

fn print_show(
    command: &DartShowCommand,
    query: &disclosures::DisclosureQuery,
    resolved: &serde_json::Value,
    disclosures_value: &serde_json::Value,
    selected: &serde_json::Value,
    document: document::DartDocument,
) {
    let list = disclosures_value
        .get("list")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    println!(
        "{}",
        crate::core::output::explained_with_raw(
            "dart show",
            "종목코드 기준 공시목록과 선택한 공시 원문 조회 결과",
            vec![
                crate::core::output::field(
                    "symbol",
                    "조회한 KRX 종목코드",
                    serde_json::json!(command.symbol),
                ),
                crate::core::output::field(
                    "resolved",
                    "종목코드를 OpenDART 고유번호로 변환한 정보",
                    resolved.clone(),
                ),
                crate::core::output::field(
                    "date_range",
                    "공시목록 검색 기간. null이면 OpenDART 기본 검색 범위",
                    serde_json::json!({
                        "from": query.bgn_de,
                        "to": query.end_de,
                    }),
                ),
                crate::core::output::field(
                    "selected_index",
                    "원문을 조회한 공시목록 항목 번호. 1부터 시작",
                    serde_json::json!(command.index),
                ),
                crate::core::output::field(
                    "selected_disclosure",
                    "원문 조회 대상으로 선택된 공시 항목",
                    selected.clone(),
                ),
                crate::core::output::field(
                    "disclosures",
                    "조회된 공시목록. 다른 항목을 열려면 --index 값을 바꾼다.",
                    list,
                ),
                crate::core::output::field(
                    "document",
                    "선택된 공시의 원문 ZIP 캐시 정보와 추출 본문",
                    serde_json::json!(document),
                ),
            ],
            serde_json::Value::Null,
        )
    );
}
