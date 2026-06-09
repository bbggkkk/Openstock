pub fn agent() -> ureq::Agent {
    let config = ureq::config::Config::builder()
        .tls_config(
            ureq::tls::TlsConfig::builder()
                .provider(ureq::tls::TlsProvider::NativeTls)
                .build(),
        )
        .build();
    ureq::Agent::new_with_config(config)
}
