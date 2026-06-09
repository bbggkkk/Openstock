pub mod account_config;
pub mod account_status;
pub mod buy;
pub mod call;
pub mod info;
pub mod login;
pub mod market;
pub mod market_history;
pub mod order_common;
pub mod order_status;
pub mod sell;
pub mod stock_info;

use crate::core::{LoginArguments, TraderApi, TraderBase};

/// 한국투자증권(KIS) API 구현체
pub struct KisApi {
    base: TraderBase,
    token: Option<String>,
}

impl KisApi {
    pub fn new() -> Self {
        KisApi {
            base: TraderBase::new("KIS", "한국투자증권", "Korea Investment & Securities"),
            token: None,
        }
    }

    pub(crate) fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }
}

impl TraderApi for KisApi {
    fn base(&self) -> &TraderBase {
        &self.base
    }

    fn login(&mut self, args: &dyn LoginArguments) -> Result<(), String> {
        let kis_args = args
            .as_any()
            .downcast_ref::<login::KisLoginArguments>()
            .ok_or("[KIS] login: 잘못된 로그인 인자 타입입니다")?;

        let token = login::login(kis_args)?;
        self.token = Some(token);
        Ok(())
    }

    fn call(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<String, String> {
        call::call(self, endpoint, params)
    }

    fn account_status(&self) -> Result<String, String> {
        account_status::account_status(self)
    }

    fn buy(
        &self,
        symbol: &str,
        qty: u64,
        price: Option<u64>,
        market: bool,
    ) -> Result<String, String> {
        buy::buy(self, symbol, qty, price, market)
    }

    fn sell(
        &self,
        symbol: &str,
        qty: u64,
        price: Option<u64>,
        market: bool,
    ) -> Result<String, String> {
        sell::sell(self, symbol, qty, price, market)
    }

    fn order_status(
        &self,
        order_no: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<String, String> {
        order_status::order_status(self, order_no, start_date, end_date)
    }

    fn market(&self, symbol: &str) -> Result<String, String> {
        market::market(self, symbol)
    }

    fn market_history(
        &self,
        symbol: &str,
        start_date: &str,
        end_date: &str,
        period: &str,
        adjusted: bool,
    ) -> Result<String, String> {
        market_history::market_history(self, symbol, start_date, end_date, period, adjusted)
    }

    fn info(&self) -> Vec<(&'static str, &'static str)> {
        info::info()
    }
}
