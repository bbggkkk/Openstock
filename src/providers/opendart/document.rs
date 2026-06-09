use crate::core::dotenv;
use encoding_rs::EUC_KR;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

const OPENDART_DOCUMENT_URL: &str = "https://opendart.fss.or.kr/api/document.xml";
const CACHE_DIR: &str = ".openstock/opendart/documents";
const DOCUMENT_RETENTION_FILES: usize = 100;
const DOCUMENT_RETENTION_BYTES: u64 = 200 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DartDocumentOptions {
    pub rcept_no: String,
    pub force: bool,
    pub max_chars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DartDocument {
    pub rcept_no: String,
    pub source: String,
    pub source_url: String,
    pub viewer_url: String,
    pub zip_path: String,
    pub cached: bool,
    pub zip_bytes: u64,
    pub files: Vec<DartDocumentFile>,
    pub text: String,
    pub text_chars: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DartDocumentFile {
    pub name: String,
    pub bytes: u64,
    pub text_chars: usize,
}

pub fn get(options: &DartDocumentOptions) -> Result<DartDocument, String> {
    validate_rcept_no(&options.rcept_no)?;
    let max_chars = options.max_chars.max(1);
    let zip_path = cache_path(&options.rcept_no);

    let (bytes, cached) = if !options.force && zip_path.exists() {
        (
            fs::read(&zip_path).map_err(|err| {
                format!(
                    "[OpenDART] 공시서류 캐시 읽기 실패 ({}): {}",
                    zip_path.display(),
                    err
                )
            })?,
            true,
        )
    } else {
        let bytes = download(&options.rcept_no)?;
        write_cache(&zip_path, &bytes)?;
        prune_cache(false)?;
        (bytes, false)
    };

    let parsed = parse_document_zip(&bytes, max_chars)?;
    let zip_bytes = bytes.len() as u64;

    Ok(DartDocument {
        rcept_no: options.rcept_no.clone(),
        source: "OpenDART".to_string(),
        source_url: OPENDART_DOCUMENT_URL.to_string(),
        viewer_url: format!(
            "https://dart.fss.or.kr/dsaf001/main.do?rcpNo={}",
            options.rcept_no
        ),
        zip_path: zip_path.display().to_string(),
        cached,
        zip_bytes,
        files: parsed.files,
        text: parsed.text,
        text_chars: parsed.text_chars,
        truncated: parsed.truncated,
    })
}

pub fn prune_cache(dry_run: bool) -> Result<crate::core::cache::CachePruneReport, String> {
    crate::core::cache::prune_snapshots(
        "opendart/documents",
        &cache_dir(),
        &[],
        &["zip"],
        DOCUMENT_RETENTION_FILES,
        DOCUMENT_RETENTION_BYTES,
        dry_run,
    )
}

fn download(rcept_no: &str) -> Result<Vec<u8>, String> {
    let key = api_key()?;
    let url = format!(
        "{}?crtfc_key={}&rcept_no={}",
        OPENDART_DOCUMENT_URL,
        percent_encode(&key),
        percent_encode(rcept_no)
    );
    let response = crate::core::http::agent()
        .get(&url)
        .header("User-Agent", "openstock/0.1")
        .call()
        .map_err(|err| format!("[OpenDART] 공시서류 원본파일 요청 실패: {}", err))?;
    let status = response.status();
    let body = response
        .into_body()
        .read_to_vec()
        .map_err(|err| format!("[OpenDART] 공시서류 원본파일 응답 읽기 실패: {}", err))?;

    if !status.is_success() {
        return Err(format!(
            "[OpenDART] 공시서류 원본파일 요청 오류 ({}): {} bytes",
            status,
            body.len()
        ));
    }

    Ok(body)
}

struct ParsedDocument {
    files: Vec<DartDocumentFile>,
    text: String,
    text_chars: usize,
    truncated: bool,
}

fn parse_document_zip(bytes: &[u8], max_chars: usize) -> Result<ParsedDocument, String> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|err| {
        let text = decode_bytes(bytes);
        format!(
            "[OpenDART] 공시서류 ZIP 파싱 실패: {} / 응답 앞부분: {}",
            err,
            text.chars().take(200).collect::<String>()
        )
    })?;

    let mut files = Vec::new();
    let mut collected = String::new();
    let mut truncated = false;

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|err| format!("[OpenDART] 공시서류 ZIP 항목 읽기 실패: {}", err))?;
        if file.is_dir() {
            continue;
        }

        let name = file.name().to_string();
        let bytes = file.size();
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)
            .map_err(|err| format!("[OpenDART] 공시서류 파일 내용 읽기 실패: {}", err))?;
        let text = if looks_like_xml(&name, &file_bytes) {
            xml_to_text(&decode_bytes(&file_bytes))
        } else {
            String::new()
        };
        let text_chars = text.chars().count();
        if !text.is_empty() && !truncated {
            append_limited(&mut collected, &text, max_chars, &mut truncated);
        }
        files.push(DartDocumentFile {
            name,
            bytes,
            text_chars,
        });
    }

    if files.is_empty() {
        return Err("[OpenDART] 공시서류 ZIP에 파일이 없습니다.".to_string());
    }

    Ok(ParsedDocument {
        files,
        text_chars: collected.chars().count(),
        text: collected,
        truncated,
    })
}

fn append_limited(target: &mut String, text: &str, max_chars: usize, truncated: &mut bool) {
    let remaining = max_chars.saturating_sub(target.chars().count());
    if remaining == 0 {
        *truncated = true;
        return;
    }
    let mut iter = text.chars();
    for ch in iter.by_ref().take(remaining) {
        target.push(ch);
    }
    if iter.next().is_some() {
        *truncated = true;
    }
}

fn xml_to_text(xml: &str) -> String {
    let xml = remove_element_blocks(xml, "style");
    let xml = remove_element_blocks(&xml, "script");
    let mut text = String::new();
    let mut inside_tag = false;
    for ch in xml.chars() {
        match ch {
            '<' => {
                inside_tag = true;
                text.push(' ');
            }
            '>' => {
                inside_tag = false;
                text.push(' ');
            }
            _ if !inside_tag => text.push(ch),
            _ => {}
        }
    }
    decode_xml_entities(&text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn remove_element_blocks(xml: &str, tag: &str) -> String {
    let mut output = String::new();
    let mut rest = xml;
    let open_pattern = format!("<{}", tag);
    let close_pattern = format!("</{}>", tag);

    loop {
        let Some(open_start) = find_ascii_case_insensitive(rest, &open_pattern) else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..open_start]);
        let after_open = &rest[open_start..];
        let Some(close_start) = find_ascii_case_insensitive(after_open, &close_pattern) else {
            break;
        };
        let after_close = close_start + close_pattern.len();
        rest = &after_open[after_close..];
    }

    output
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn decode_bytes(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(value) => value.to_string(),
        Err(_) => {
            let (decoded, _, _) = EUC_KR.decode(bytes);
            decoded.into_owned()
        }
    }
}

fn looks_like_xml(name: &str, bytes: &[u8]) -> bool {
    name.to_ascii_lowercase().ends_with(".xml")
        || decode_bytes(&bytes[..bytes.len().min(100)])
            .trim_start()
            .starts_with('<')
}

fn decode_xml_entities(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn write_cache(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("[OpenDART] 공시서류 캐시 디렉터리 생성 실패: {}", err))?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).map_err(|err| format!("[OpenDART] 공시서류 캐시 쓰기 실패: {}", err))?;
    fs::rename(&tmp, path).map_err(|err| format!("[OpenDART] 공시서류 캐시 교체 실패: {}", err))?;
    Ok(())
}

fn cache_path(rcept_no: &str) -> PathBuf {
    cache_dir().join(format!("{}.zip", rcept_no))
}

fn cache_dir() -> PathBuf {
    PathBuf::from(CACHE_DIR)
}

fn api_key() -> Result<String, String> {
    dotenv::read_env(Path::new(".env"))
        .get("OPENDART_API_KEY")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or("OPENDART_API_KEY가 .env에 없습니다.".to_string())
}

fn validate_rcept_no(value: &str) -> Result<(), String> {
    if value.len() == 14 && value.chars().all(|ch| ch.is_ascii_digit()) {
        return Ok(());
    }
    Err("rcept_no는 14자리 숫자 접수번호여야 합니다.".to_string())
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_receipt_number() {
        assert!(validate_rcept_no("20260608800918").is_ok());
        assert!(validate_rcept_no("2026060880091").is_err());
        assert!(validate_rcept_no("2026060880091A").is_err());
    }

    #[test]
    fn extracts_readable_text_from_xml() {
        let xml = r#"<DOCUMENT><STYLE>.xforms { color: red; }</STYLE><TITLE>사업보고서</TITLE><BODY>매출 &amp; 이익 <TABLE><TR><TD>100</TD></TR></TABLE></BODY></DOCUMENT>"#;
        let text = xml_to_text(xml);
        assert!(text.contains("사업보고서"));
        assert!(text.contains("매출 & 이익"));
        assert!(text.contains("100"));
        assert!(!text.contains("xforms"));
        assert!(!text.contains("<TITLE>"));
    }

    #[test]
    fn appends_text_with_character_limit() {
        let mut target = String::new();
        let mut truncated = false;
        append_limited(&mut target, "가나다라마", 3, &mut truncated);
        assert_eq!(target, "가나다");
        assert!(truncated);
    }
}
