use super::trader::TraderApi;

/// API 구현체를 등록하고 관리하는 레지스트리
pub struct ApiRegistry {
    apis: Vec<Box<dyn TraderApi>>,
}

impl ApiRegistry {
    /// 빈 레지스트리 생성
    pub fn new() -> Self {
        Self { apis: Vec::new() }
    }

    /// API 구현체 등록
    pub fn register(&mut self, api: Box<dyn TraderApi>) {
        self.apis.push(api);
    }

    /// 등록된 모든 API 목록 반환
    pub fn list(&self) -> &[Box<dyn TraderApi>] {
        &self.apis
    }
}
