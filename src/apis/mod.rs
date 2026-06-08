#[path = "kis/main.rs"]
pub mod kis;

use crate::core::ApiRegistry;

/// 기본 증권사 API를 등록한 레지스트리 생성
pub fn create_default_registry() -> ApiRegistry {
    let mut registry = ApiRegistry::new();

    // 한국투자증권 KIS 등록
    registry.register(Box::new(kis::KisApi::new()));

    registry
}
