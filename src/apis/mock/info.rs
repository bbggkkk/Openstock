/// MockApi 정보 구성
pub fn info(id: &'static str, name: &'static str, description: &'static str) -> Vec<(&'static str, &'static str)> {
    vec![
        ("id", id),
        ("name", name),
        ("description", description),
        ("version", "0.0.0"),
        ("status", "mock"),
    ]
}
