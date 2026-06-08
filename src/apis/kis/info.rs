/// KisApi 표준 정보 반환
pub fn info() -> Vec<(&'static str, &'static str)> {
    vec![
        ("id", "KIS"),
        ("name", "한국투자증권"),
        ("description", "Korea Investment & Securities"),
        ("version", "1.0"),
        ("status", "available"),
    ]
}
