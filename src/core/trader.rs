/// 증권사 기본 정보
pub struct TraderBase {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

impl TraderBase {
    pub fn new(id: &'static str, name: &'static str, description: &'static str) -> Self {
        Self {
            id,
            name,
            description,
        }
    }
}

/// 증권사별 로그인에 필요한 인자들의 공통 트레이트
pub trait LoginArguments {
    /// .env 파일에 저장할 토큰 키 이름 (예: "KIS_ACCESS_TOKEN")
    fn token_env_key(&self) -> &'static str;

    /// 구체적인 타입으로 다운캐스팅하기 위한 Any 변환
    fn as_any(&self) -> &dyn std::any::Any;
}

/// 증권사 API가 구현해야 하는 공통 트레이트
pub trait TraderApi {
    /// 기본 정보 반환
    fn base(&self) -> &TraderBase;

    /// 증권사 ID
    fn id(&self) -> &'static str {
        self.base().id
    }

    /// 증권사 이름
    fn name(&self) -> &'static str {
        self.base().name
    }

    /// 증권사 설명
    fn description(&self) -> &'static str {
        self.base().description
    }

    /// 로그인 (토큰 발급 등 인증 처리)
    fn login(&mut self, args: &dyn LoginArguments) -> Result<(), String>;

    /// API 호출 (endpoint: API 경로, params: 요청 파라미터)
    fn call(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<String, String>;

    /// 계좌 상태 조회 (잔액, 보유종목 등)
    fn account_status(&self) -> Result<String, String>;

    /// 매수 주문
    fn buy(
        &self,
        symbol: &str,
        qty: u64,
        price: Option<u64>,
        market: bool,
    ) -> Result<String, String>;

    /// 매도 주문
    fn sell(
        &self,
        symbol: &str,
        qty: u64,
        price: Option<u64>,
        market: bool,
    ) -> Result<String, String>;

    /// 주문/체결 조회
    fn order_status(
        &self,
        order_no: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<String, String>;

    /// 종목 시장 정보 조회
    fn market(&self, symbol: &str) -> Result<String, String>;

    /// 증권사 정보 (키-값 쌍)
    fn info(&self) -> Vec<(&'static str, &'static str)>;
}
