use crate::core::dotenv;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const OPENDART_CORP_CODE_URL: &str = "https://opendart.fss.or.kr/api/corpCode.xml";
const CACHE_DIR: &str = ".openstock/opendart";
const CORP_CODES_FILE: &str = "corp_codes.json";
const META_FILE: &str = "corp_codes_meta.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DartCorpCode {
    pub corp_code: String,
    pub corp_name: String,
    pub stock_code: String,
    pub modify_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DartCorpCodeMeta {
    pub source: String,
    pub source_url: String,
    pub date: String,
    pub refreshed_at: String,
    pub total_count: usize,
    pub listed_count: usize,
}

#[derive(Debug, Clone)]
pub struct DartCorpCodeSnapshot {
    pub corps: Vec<DartCorpCode>,
    pub meta: DartCorpCodeMeta,
    pub refreshed: bool,
}

pub fn load_or_refresh(force: bool) -> Result<DartCorpCodeSnapshot, String> {
    if !force {
        if let Some(snapshot) = load_today_cache()? {
            return Ok(snapshot);
        }
    }

    match refresh() {
        Ok(snapshot) => Ok(snapshot),
        Err(err) => {
            if let Some(snapshot) = load_any_cache()? {
                return Ok(DartCorpCodeSnapshot {
                    refreshed: false,
                    ..snapshot
                });
            }
            Err(err)
        }
    }
}

pub fn load_cached() -> Result<Option<DartCorpCodeSnapshot>, String> {
    load_any_cache()
}

pub fn refresh() -> Result<DartCorpCodeSnapshot, String> {
    let key = api_key()?;
    let url = format!("{}?crtfc_key={}", OPENDART_CORP_CODE_URL, key);
    let response = crate::core::http::agent()
        .get(&url)
        .header("User-Agent", "openstock/0.1")
        .call()
        .map_err(|err| format!("[OpenDART] 고유번호 다운로드 실패: {}", err))?;
    let status = response.status();
    let body = response
        .into_body()
        .read_to_vec()
        .map_err(|err| format!("[OpenDART] 고유번호 응답 읽기 실패: {}", err))?;

    if !status.is_success() {
        return Err(format!(
            "[OpenDART] 고유번호 다운로드 오류 ({}): {} bytes",
            status,
            body.len()
        ));
    }

    let all_corps = parse_corp_code_zip(&body)?;
    let total_count = all_corps.len();
    let mut listed = all_corps
        .into_iter()
        .filter(|corp| !corp.stock_code.trim().is_empty())
        .collect::<Vec<_>>();
    listed.sort_by(|a, b| {
        a.stock_code
            .cmp(&b.stock_code)
            .then_with(|| a.corp_code.cmp(&b.corp_code))
    });

    let meta = DartCorpCodeMeta {
        source: "OpenDART".to_string(),
        source_url: OPENDART_CORP_CODE_URL.to_string(),
        date: current_utc_date_string(),
        refreshed_at: current_utc_datetime_string(),
        total_count,
        listed_count: listed.len(),
    };
    write_cache(&listed, &meta)?;

    Ok(DartCorpCodeSnapshot {
        corps: listed,
        meta,
        refreshed: true,
    })
}

pub fn find_by_stock_code<'a>(corps: &'a [DartCorpCode], symbol: &str) -> Option<&'a DartCorpCode> {
    corps.iter().find(|corp| corp.stock_code == symbol)
}

fn api_key() -> Result<String, String> {
    dotenv::read_env(Path::new(".env"))
        .get("OPENDART_API_KEY")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or("OPENDART_API_KEY가 .env에 없습니다.".to_string())
}

fn parse_corp_code_zip(bytes: &[u8]) -> Result<Vec<DartCorpCode>, String> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|err| {
        let text = String::from_utf8_lossy(bytes);
        format!(
            "[OpenDART] 고유번호 ZIP 파싱 실패: {} / 응답 앞부분: {}",
            err,
            text.chars().take(200).collect::<String>()
        )
    })?;
    let mut xml = String::new();
    archive
        .by_name("CORPCODE.xml")
        .map_err(|err| format!("[OpenDART] CORPCODE.xml 파일을 찾지 못했습니다: {}", err))?
        .read_to_string(&mut xml)
        .map_err(|err| format!("[OpenDART] CORPCODE.xml 읽기 실패: {}", err))?;
    parse_corps_xml(&xml)
}

fn parse_corps_xml(xml: &str) -> Result<Vec<DartCorpCode>, String> {
    let mut corps = Vec::new();
    let mut rest = xml;

    while let Some(start) = rest.find("<list>") {
        let item_start = start + "<list>".len();
        let Some(end) = rest[item_start..].find("</list>") else {
            break;
        };
        let item = &rest[item_start..item_start + end];
        let corp_code = extract_tag(item, "corp_code").unwrap_or_default();
        let corp_name = extract_tag(item, "corp_name").unwrap_or_default();
        if !corp_code.is_empty() && !corp_name.is_empty() {
            corps.push(DartCorpCode {
                corp_code,
                corp_name,
                stock_code: extract_tag(item, "stock_code").unwrap_or_default(),
                modify_date: extract_tag(item, "modify_date").unwrap_or_default(),
            });
        }
        rest = &rest[item_start + end + "</list>".len()..];
    }

    if corps.is_empty() {
        return Err("[OpenDART] 고유번호 XML에서 회사를 찾지 못했습니다.".to_string());
    }
    Ok(corps)
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(decode_xml_entities(xml[start..start + end].trim()))
}

fn decode_xml_entities(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn write_cache(corps: &[DartCorpCode], meta: &DartCorpCodeMeta) -> Result<(), String> {
    let dir = cache_dir();
    fs::create_dir_all(&dir)
        .map_err(|err| format!("[OpenDART] 캐시 디렉터리 생성 실패: {}", err))?;
    write_json_atomic(&dir.join(CORP_CODES_FILE), corps)?;
    write_json_atomic(&dir.join(META_FILE), meta)?;
    Ok(())
}

fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    let json = serde_json::to_string_pretty(value)
        .map_err(|err| format!("[OpenDART] 캐시 JSON 직렬화 실패: {}", err))?;
    fs::write(&tmp, json).map_err(|err| format!("[OpenDART] 캐시 임시 파일 쓰기 실패: {}", err))?;
    fs::rename(&tmp, path).map_err(|err| format!("[OpenDART] 캐시 파일 교체 실패: {}", err))?;
    Ok(())
}

fn load_today_cache() -> Result<Option<DartCorpCodeSnapshot>, String> {
    let Some(snapshot) = load_any_cache()? else {
        return Ok(None);
    };
    if snapshot.meta.date == current_utc_date_string() {
        return Ok(Some(snapshot));
    }
    Ok(None)
}

fn load_any_cache() -> Result<Option<DartCorpCodeSnapshot>, String> {
    let dir = cache_dir();
    let corps_path = dir.join(CORP_CODES_FILE);
    let meta_path = dir.join(META_FILE);
    if !corps_path.exists() || !meta_path.exists() {
        return Ok(None);
    }

    Ok(Some(DartCorpCodeSnapshot {
        corps: read_json(&corps_path)?,
        meta: read_json(&meta_path)?,
        refreshed: false,
    }))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|err| {
        format!(
            "[OpenDART] 캐시 파일 읽기 실패 ({}): {}",
            path.display(),
            err
        )
    })?;
    serde_json::from_str(&text).map_err(|err| {
        format!(
            "[OpenDART] 캐시 JSON 파싱 실패 ({}): {}",
            path.display(),
            err
        )
    })
}

fn cache_dir() -> PathBuf {
    PathBuf::from(CACHE_DIR)
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
