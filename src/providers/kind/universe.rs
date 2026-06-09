use crate::core::stock::{
    infer_stock_kind, normalize_kind_market, validate_symbol, Stock, StockId, StockKind,
    StockMarket, StockSource,
};
use encoding_rs::EUC_KR;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const KIND_UNIVERSE_URL: &str =
    "https://kind.krx.co.kr/corpgeneral/corpList.do?method=download&searchType=13";
const CACHE_DIR: &str = ".openstock/universe/kind";
const LATEST_FILE: &str = "latest.json";
const META_FILE: &str = "meta.json";
const SNAPSHOT_RETENTION_FILES: usize = 7;
const SNAPSHOT_RETENTION_BYTES: u64 = 25 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseMeta {
    pub source: String,
    pub source_url: String,
    pub date: String,
    pub refreshed_at: String,
    pub stock_count: usize,
    pub counts_by_market: BTreeMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct UniverseSnapshot {
    pub stocks: Vec<Stock>,
    pub meta: UniverseMeta,
    pub refreshed: bool,
}

pub fn load_or_refresh(force: bool) -> Result<UniverseSnapshot, String> {
    if !force {
        if let Some(snapshot) = load_today_cache()? {
            return Ok(snapshot);
        }
    }

    match refresh() {
        Ok(snapshot) => Ok(snapshot),
        Err(err) => {
            if let Some(snapshot) = load_any_cache()? {
                return Ok(UniverseSnapshot {
                    refreshed: false,
                    ..snapshot
                });
            }
            Err(err)
        }
    }
}

pub fn refresh() -> Result<UniverseSnapshot, String> {
    let body = download()?;
    let html = decode_euc_kr(&body);
    let mut stocks = parse_kind_html(&html)?;
    stocks.sort_by(|a, b| {
        a.market
            .cmp(&b.market)
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.id.symbol.cmp(&b.id.symbol))
    });

    let today = current_utc_date_string();
    for stock in &mut stocks {
        stock.updated_at = Some(today.clone());
    }

    let meta = UniverseMeta {
        source: "KIND".to_string(),
        source_url: KIND_UNIVERSE_URL.to_string(),
        date: today.clone(),
        refreshed_at: current_utc_datetime_string(),
        stock_count: stocks.len(),
        counts_by_market: counts_by_market(&stocks),
    };

    write_cache(&stocks, &meta)?;

    Ok(UniverseSnapshot {
        stocks,
        meta,
        refreshed: true,
    })
}

pub fn load_cached() -> Result<Option<UniverseSnapshot>, String> {
    load_any_cache()
}

pub fn prune_cache(dry_run: bool) -> Result<crate::core::cache::CachePruneReport, String> {
    crate::core::cache::prune_json_snapshots(
        "universe/kind",
        &cache_dir(),
        &[LATEST_FILE, META_FILE],
        SNAPSHOT_RETENTION_FILES,
        SNAPSHOT_RETENTION_BYTES,
        dry_run,
    )
}

pub fn chunk_stocks(stocks: &[Stock], size: usize) -> Vec<serde_json::Value> {
    let size = size.max(1);
    let mut groups = BTreeMap::<(StockMarket, StockKind), Vec<&Stock>>::new();
    for stock in stocks {
        groups
            .entry((stock.market, stock.kind))
            .or_default()
            .push(stock);
    }

    let mut chunks = Vec::new();
    for ((market, kind), group) in groups {
        for (index, chunk) in group.chunks(size).enumerate() {
            let first = chunk.first().copied();
            let last = chunk.last().copied();
            chunks.push(serde_json::json!({
                "chunk_id": format!("{}:{}:{:04}", market.as_str(), kind.as_str(), index + 1),
                "index": index + 1,
                "size": size,
                "count": chunk.len(),
                "start_symbol": first.map(|stock| stock.id.symbol.as_str()),
                "end_symbol": last.map(|stock| stock.id.symbol.as_str()),
                "market": market.as_str(),
                "kind": kind.as_str(),
            }));
        }
    }

    chunks
}

fn download() -> Result<Vec<u8>, String> {
    let response = crate::core::http::agent()
        .get(KIND_UNIVERSE_URL)
        .header("User-Agent", "openstock/0.1")
        .call()
        .map_err(|err| format!("[KIND] 상장법인목록 다운로드 실패: {}", err))?;
    let status = response.status();
    let body = response
        .into_body()
        .read_to_vec()
        .map_err(|err| format!("[KIND] 상장법인목록 응답 읽기 실패: {}", err))?;

    if !status.is_success() {
        return Err(format!(
            "[KIND] 상장법인목록 다운로드 오류 ({}): {} bytes",
            status,
            body.len()
        ));
    }

    Ok(body)
}

fn decode_euc_kr(bytes: &[u8]) -> String {
    let (decoded, _, _) = EUC_KR.decode(bytes);
    decoded.into_owned()
}

fn parse_kind_html(html: &str) -> Result<Vec<Stock>, String> {
    let mut rows = Vec::new();
    let mut rest = html;

    while let Some(start) = find_ascii_case_insensitive(rest, "<tr") {
        let row_start = &rest[start..];
        let Some(open_end) = row_start.find('>') else {
            break;
        };
        let row_body_start = open_end + 1;
        let Some(close) = find_ascii_case_insensitive(&row_start[row_body_start..], "</tr>") else {
            break;
        };
        let row_body = &row_start[row_body_start..row_body_start + close];
        let cells = extract_cells(row_body);
        if cells.len() >= 10 && cells[0] != "회사명" {
            if let Some(stock) = row_to_stock(&cells) {
                rows.push(stock);
            }
        }
        rest = &row_start[row_body_start + close + "</tr>".len()..];
    }

    if rows.is_empty() {
        return Err("[KIND] 상장법인목록에서 종목을 찾지 못했습니다.".to_string());
    }

    Ok(rows)
}

fn row_to_stock(cells: &[String]) -> Option<Stock> {
    let name = cells.first()?.trim();
    let market = cells.get(1)?.trim();
    let symbol = cells.get(2)?.trim();
    let industry = empty_to_none(cells.get(3).map(String::as_str).unwrap_or(""));
    let main_product = empty_to_none(cells.get(4).map(String::as_str).unwrap_or(""));

    if name.is_empty() || validate_symbol(symbol).is_err() {
        return None;
    }

    Some(Stock {
        id: StockId {
            symbol: symbol.to_string(),
            isin: None,
            reuters_code: None,
        },
        name: name.to_string(),
        market: normalize_kind_market(market),
        kind: infer_stock_kind(name, industry.as_deref(), main_product.as_deref()),
        industry,
        sector: None,
        main_product,
        listed_at: empty_to_none(cells.get(5).map(String::as_str).unwrap_or("")),
        fiscal_month: empty_to_none(cells.get(6).map(String::as_str).unwrap_or("")),
        homepage: empty_to_none(cells.get(8).map(String::as_str).unwrap_or("")),
        region: empty_to_none(cells.get(9).map(String::as_str).unwrap_or("")),
        source: StockSource::Kind,
        source_url: Some(KIND_UNIVERSE_URL.to_string()),
        updated_at: None,
    })
}

fn extract_cells(row: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut rest = row;

    loop {
        let td = find_ascii_case_insensitive(rest, "<td");
        let th = find_ascii_case_insensitive(rest, "<th");
        let Some(start) = (match (td, th) {
            (Some(td), Some(th)) => Some(td.min(th)),
            (Some(td), None) => Some(td),
            (None, Some(th)) => Some(th),
            (None, None) => None,
        }) else {
            break;
        };

        let cell_start = &rest[start..];
        let Some(open_end) = cell_start.find('>') else {
            break;
        };
        let close_tag = if cell_start.to_ascii_lowercase().starts_with("<th") {
            "</th>"
        } else {
            "</td>"
        };
        let body_start = open_end + 1;
        let Some(close) = find_ascii_case_insensitive(&cell_start[body_start..], close_tag) else {
            break;
        };
        let body = &cell_start[body_start..body_start + close];
        cells.push(clean_cell(body));
        rest = &cell_start[body_start + close + close_tag.len()..];
    }

    cells
}

fn clean_cell(value: &str) -> String {
    let mut text = String::new();
    let mut inside_tag = false;

    for ch in value.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                text.push(' ');
            }
            _ if !inside_tag => text.push(ch),
            _ => {}
        }
    }

    decode_entities(&text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn empty_to_none(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn write_cache(stocks: &[Stock], meta: &UniverseMeta) -> Result<(), String> {
    let dir = cache_dir();
    fs::create_dir_all(&dir).map_err(|err| format!("[KIND] 캐시 디렉터리 생성 실패: {}", err))?;
    write_json_atomic(&dir.join(LATEST_FILE), stocks)?;
    write_json_atomic(&dir.join(META_FILE), meta)?;
    write_json_atomic(&dir.join(format!("{}.json", meta.date)), stocks)?;
    prune_cache(false)?;
    Ok(())
}

fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    let json = serde_json::to_string_pretty(value)
        .map_err(|err| format!("[KIND] 캐시 JSON 직렬화 실패: {}", err))?;
    fs::write(&tmp, json).map_err(|err| format!("[KIND] 캐시 임시 파일 쓰기 실패: {}", err))?;
    fs::rename(&tmp, path).map_err(|err| format!("[KIND] 캐시 파일 교체 실패: {}", err))?;
    Ok(())
}

fn load_today_cache() -> Result<Option<UniverseSnapshot>, String> {
    let Some(snapshot) = load_any_cache()? else {
        return Ok(None);
    };
    if snapshot.meta.date == current_utc_date_string() {
        return Ok(Some(snapshot));
    }
    Ok(None)
}

fn load_any_cache() -> Result<Option<UniverseSnapshot>, String> {
    let dir = cache_dir();
    let latest_path = dir.join(LATEST_FILE);
    let meta_path = dir.join(META_FILE);
    if !latest_path.exists() || !meta_path.exists() {
        return Ok(None);
    }

    let stocks = read_json::<Vec<Stock>>(&latest_path)?;
    let meta = read_json::<UniverseMeta>(&meta_path)?;
    Ok(Some(UniverseSnapshot {
        stocks,
        meta,
        refreshed: false,
    }))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("[KIND] 캐시 파일 읽기 실패 ({}): {}", path.display(), err))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("[KIND] 캐시 JSON 파싱 실패 ({}): {}", path.display(), err))
}

fn cache_dir() -> PathBuf {
    PathBuf::from(CACHE_DIR)
}

fn counts_by_market(stocks: &[Stock]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for stock in stocks {
        *counts.entry(stock.market.as_str().to_string()).or_insert(0) += 1;
    }
    counts
}

fn current_utc_date_string() -> String {
    let now_seconds = current_unix_seconds();
    let days = now_seconds.div_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn current_utc_datetime_string() -> String {
    let now_seconds = current_unix_seconds();
    let days = now_seconds.div_euclid(86_400);
    let seconds_of_day = now_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };

    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::stock::{StockKind, StockMarket};

    #[test]
    fn parses_kind_html_table() {
        let html = r#"
        <table>
          <tr><th>회사명</th><th>시장구분</th><th>종목코드</th><th>업종</th><th>주요제품</th><th>상장일</th><th>결산월</th><th>대표자명</th><th>홈페이지</th><th>지역</th></tr>
          <tr><td>삼성전자</td><td>유가</td><td>005930</td><td>통신 및 방송 장비 제조업</td><td>반도체</td><td>1975-06-11</td><td>12월</td><td>대표</td><td>https://www.samsung.com</td><td>경기도</td></tr>
          <tr><td>삼성전자우</td><td>유가</td><td>005935</td><td>통신 및 방송 장비 제조업</td><td>반도체</td><td>1989-01-01</td><td>12월</td><td>대표</td><td></td><td>경기도</td></tr>
        </table>
        "#;

        let stocks = parse_kind_html(html).unwrap();
        assert_eq!(stocks.len(), 2);
        assert_eq!(stocks[0].id.symbol, "005930");
        assert_eq!(stocks[0].market, StockMarket::Kospi);
        assert_eq!(stocks[0].kind, StockKind::CommonStock);
        assert_eq!(stocks[1].kind, StockKind::PreferredStock);
    }
}
