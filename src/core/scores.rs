use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockScoreStore {
    pub version: u32,
    pub scores: BTreeMap<String, StockScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockScore {
    pub symbol: String,
    pub score: u8,
    pub updated_at_unix: u64,
}

impl Default for StockScoreStore {
    fn default() -> Self {
        Self {
            version: 1,
            scores: BTreeMap::new(),
        }
    }
}

pub fn set(symbol: &str, score: u16) -> Result<StockScore, String> {
    validate_score(score)?;
    let symbol = normalize_symbol(symbol)?;
    let mut store = load()?;
    let record = StockScore {
        symbol: symbol.clone(),
        score: score as u8,
        updated_at_unix: current_unix_timestamp(),
    };
    store.scores.insert(symbol, record.clone());
    save(&store)?;
    Ok(record)
}

pub fn get(symbol: &str) -> Result<Option<StockScore>, String> {
    let symbol = normalize_symbol(symbol)?;
    Ok(load()?.scores.get(&symbol).cloned())
}

pub fn list() -> Result<Vec<StockScore>, String> {
    let mut scores = load()?.scores.into_values().collect::<Vec<_>>();
    scores.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.symbol.cmp(&b.symbol)));
    Ok(scores)
}

pub fn delete(symbol: &str) -> Result<Option<StockScore>, String> {
    let symbol = normalize_symbol(symbol)?;
    let mut store = load()?;
    let removed = store.scores.remove(&symbol);
    if removed.is_some() {
        save(&store)?;
    }
    Ok(removed)
}

pub fn path() -> std::path::PathBuf {
    crate::core::paths::score_file()
}

fn load() -> Result<StockScoreStore, String> {
    let path = path();
    if !path.exists() {
        return Ok(StockScoreStore::default());
    }
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("종목 점수 파일 읽기 실패 ({}): {}", path.display(), err))?;
    serde_json::from_str(&text).map_err(|err| {
        format!(
            "종목 점수 파일 JSON 파싱 실패 ({}): {}",
            path.display(),
            err
        )
    })
}

fn save(store: &StockScoreStore) -> Result<(), String> {
    let path = path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("설정 디렉터리 생성 실패 ({}): {}", parent.display(), err))?;
    }
    let text = serde_json::to_string_pretty(store)
        .map_err(|err| format!("종목 점수 JSON 직렬화 실패: {}", err))?;
    write_atomic(&path, &text)
}

fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, text).map_err(|err| {
        format!(
            "종목 점수 임시 파일 쓰기 실패 ({}): {}",
            tmp_path.display(),
            err
        )
    })?;
    fs::rename(&tmp_path, path).map_err(|err| {
        format!(
            "종목 점수 파일 교체 실패 ({} -> {}): {}",
            tmp_path.display(),
            path.display(),
            err
        )
    })
}

fn normalize_symbol(symbol: &str) -> Result<String, String> {
    let symbol = symbol.trim().to_ascii_uppercase();
    crate::core::stock::validate_symbol(&symbol)?;
    Ok(symbol)
}

fn validate_score(score: u16) -> Result<(), String> {
    if score <= 100 {
        Ok(())
    } else {
        Err("종목 점수는 0 이상 100 이하이어야 합니다.".to_string())
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_symbol_to_uppercase() {
        assert_eq!(normalize_symbol(" a123 ").unwrap(), "A123");
    }

    #[test]
    fn accepts_score_bounds() {
        assert!(validate_score(0).is_ok());
        assert!(validate_score(100).is_ok());
        assert!(validate_score(101).is_err());
    }
}
