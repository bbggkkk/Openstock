use crate::core::dotenv;
use std::path::Path;

pub(crate) struct AccountConfig {
    pub(crate) number: String,
    pub(crate) product_code: String,
}

impl AccountConfig {
    pub(crate) fn full_name(&self) -> String {
        format!("{}-{}", self.number, self.product_code)
    }
}

pub(crate) fn read_account_config() -> Result<AccountConfig, String> {
    let env = dotenv::read_env(Path::new(".env"));

    if let Some(account) = env
        .get("KIS_ACCOUNT")
        .filter(|value| !value.trim().is_empty())
    {
        let (number, product_code) = split_account(account.trim());
        return Ok(AccountConfig {
            number: number.to_string(),
            product_code: product_code.to_string(),
        });
    }

    if let Some(number) = env.get("KIS_CANO").filter(|value| !value.trim().is_empty()) {
        return Ok(AccountConfig {
            number: number.trim().to_string(),
            product_code: env
                .get("KIS_ACNT_PRDT_CD")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .unwrap_or("01")
                .to_string(),
        });
    }

    Err("계좌 설정이 없습니다. .env에 KIS_ACCOUNT=12345678-01 또는 KIS_CANO/KIS_ACNT_PRDT_CD를 설정하세요.".to_string())
}

fn split_account(account: &str) -> (&str, &str) {
    account.split_once('-').unwrap_or((account, "01"))
}
