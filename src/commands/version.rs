pub fn handle_version() {
    println!(
        "{}",
        crate::core::output::explained(
            "version",
            "openstock CLI 버전 정보",
            vec![
                crate::core::output::field(
                    "name",
                    "프로그램 이름",
                    serde_json::json!(env!("CARGO_PKG_NAME")),
                ),
                crate::core::output::field(
                    "version",
                    "현재 실행 중인 openstock 버전",
                    serde_json::json!(env!("CARGO_PKG_VERSION")),
                ),
            ],
        )
    );
}
