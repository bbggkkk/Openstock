use crate::apis::kis::order_common::{self, OrderSide};
use crate::apis::kis::KisApi;

pub fn buy(
    api: &KisApi,
    symbol: &str,
    qty: u64,
    price: Option<u64>,
    market: bool,
) -> Result<String, String> {
    order_common::order_cash(api, OrderSide::Buy, symbol.trim(), qty, price, market)
}
