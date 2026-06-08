use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// .env 파일을 읽어 key-value 맵으로 반환한다.
pub fn read_env(path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return map,
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}

/// .env 파일에 key-value를 저장한다. (기존 값은 유지, 있으면 갱신)
pub fn write_env(path: &Path, key: &str, value: &str) -> Result<(), String> {
    let mut map = read_env(path);
    map.insert(key.to_string(), value.to_string());

    let mut content = String::new();
    // 이미 기존 파일이 있으면 주석과 빈 줄을 유지하기 위해 원본을 우선 쓰고,
    // 새 키는 마지막에 추가하는 전략. 단순화를 위해 전체를 다시 쓴다.
    for (k, v) in &map {
        content.push_str(&format!("{}={}\n", k, v));
    }
    fs::write(path, content).map_err(|e| format!(".env 쓰기 실패: {}", e))
}
