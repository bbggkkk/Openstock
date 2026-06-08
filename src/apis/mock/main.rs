pub mod info;
pub mod login;

use crate::core::{TraderApi, TraderBase, LoginArguments};

/// 가상(Mock) 증권사 API 구현체 — 테스트 및 개발용
pub struct MockApi {
    base: TraderBase,
    token: Option<String>,
}

impl MockApi {
    pub fn new(id: &'static str, name: &'static str, description: &'static str) -> Self {
        MockApi {
            base: TraderBase::new(id, name, description),
            token: None,
        }
    }
}

impl Default for MockApi {
    fn default() -> Self {
        Self::new("MOCK", "Mock", "가상 증권사 API (개발/테스트용)")
    }
}

impl TraderApi for MockApi {
    fn base(&self) -> &TraderBase {
        &self.base
    }

    fn login(&mut self, args: &dyn LoginArguments) -> Result<(), String> {
        let _mock_args = args
            .as_any()
            .downcast_ref::<login::MockLoginArguments>()
            .ok_or("[Mock] login: 잘못된 로그인 인자 타입입니다")?;

        self.token = Some("mock-token".to_string());
        println!("[Mock] 로그인 성공");
        Ok(())
    }

    fn call(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<String, String> {
        let token = self.token.as_ref().ok_or("로그인이 필요합니다")?;
        println!("[Mock] API 호출: {} (토큰: {})", endpoint, token);
        for (k, v) in params {
            println!("[Mock]   {} = {}", k, v);
        }
        Ok(format!("{{\"endpoint\":\"{}\",\"status\":\"mock\"}}", endpoint))
    }

    fn info(&self) -> Vec<(&'static str, &'static str)> {
        info::info(self.base.id, self.base.name, self.base.description)
    }
}
