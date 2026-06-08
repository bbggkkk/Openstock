use crate::core::LoginArguments;

/// Mock 로그인 인자 (항상 성공)
pub struct MockLoginArguments;

impl LoginArguments for MockLoginArguments {
    fn token_env_key(&self) -> &'static str {
        "MOCK_ACCESS_TOKEN"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
