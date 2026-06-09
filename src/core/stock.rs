use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stock {
    pub id: StockId,
    pub name: String,
    pub market: StockMarket,
    pub kind: StockKind,
    pub industry: Option<String>,
    pub sector: Option<String>,
    pub main_product: Option<String>,
    pub listed_at: Option<String>,
    pub fiscal_month: Option<String>,
    pub homepage: Option<String>,
    pub region: Option<String>,
    pub source: StockSource,
    pub source_url: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockId {
    pub symbol: String,
    pub isin: Option<String>,
    pub reuters_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockMarket {
    Kospi,
    Kosdaq,
    Konex,
    Other,
    Unknown,
}

impl StockMarket {
    pub fn as_str(&self) -> &'static str {
        match self {
            StockMarket::Kospi => "KOSPI",
            StockMarket::Kosdaq => "KOSDAQ",
            StockMarket::Konex => "KONEX",
            StockMarket::Other => "OTHER",
            StockMarket::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StockKind {
    CommonStock,
    PreferredStock,
    Etf,
    Etn,
    Reit,
    Spac,
    Other,
    Unknown,
}

impl StockKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StockKind::CommonStock => "common_stock",
            StockKind::PreferredStock => "preferred_stock",
            StockKind::Etf => "etf",
            StockKind::Etn => "etn",
            StockKind::Reit => "reit",
            StockKind::Spac => "spac",
            StockKind::Other => "other",
            StockKind::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockSource {
    Kind,
    Naver,
    Kis,
    Manual,
}

impl Stock {
    #[allow(dead_code)]
    pub fn new(symbol: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: StockId {
                symbol: symbol.into(),
                isin: None,
                reuters_code: None,
            },
            name: name.into(),
            market: StockMarket::Unknown,
            kind: StockKind::Unknown,
            industry: None,
            sector: None,
            main_product: None,
            listed_at: None,
            fiscal_month: None,
            homepage: None,
            region: None,
            source: StockSource::Manual,
            source_url: None,
            updated_at: None,
        }
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        validate_symbol(&self.id.symbol)?;
        if self.name.trim().is_empty() {
            return Err("종목명은 비어 있을 수 없습니다.".to_string());
        }
        Ok(())
    }
}

pub fn validate_symbol(symbol: &str) -> Result<(), String> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Err("종목코드는 비어 있을 수 없습니다.".to_string());
    }
    if symbol.len() > 12 {
        return Err("종목코드는 12자를 넘을 수 없습니다.".to_string());
    }
    if !symbol.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err("종목코드는 영문 대문자/소문자와 숫자만 허용합니다.".to_string());
    }
    Ok(())
}

pub fn normalize_kind_market(value: &str) -> StockMarket {
    match value.trim() {
        "유가" | "KOSPI" | "kospi" => StockMarket::Kospi,
        "코스닥" | "KOSDAQ" | "kosdaq" => StockMarket::Kosdaq,
        "코넥스" | "KONEX" | "konex" => StockMarket::Konex,
        "" => StockMarket::Unknown,
        _ => StockMarket::Other,
    }
}

pub fn infer_stock_kind(
    name: &str,
    industry: Option<&str>,
    main_product: Option<&str>,
) -> StockKind {
    let text = format!(
        "{} {} {}",
        name,
        industry.unwrap_or(""),
        main_product.unwrap_or("")
    )
    .to_ascii_uppercase();

    if text.contains(" ETF")
        || text.starts_with("ETF ")
        || text.contains("KODEX")
        || text.contains("TIGER")
    {
        return StockKind::Etf;
    }
    if text.contains(" ETN") || text.starts_with("ETN ") {
        return StockKind::Etn;
    }
    if text.contains("리츠") || text.contains("REIT") {
        return StockKind::Reit;
    }
    if text.contains("스팩") || text.contains("SPAC") {
        return StockKind::Spac;
    }
    if name.ends_with('우') || name.contains("우B") || name.contains("우C") {
        return StockKind::PreferredStock;
    }

    StockKind::CommonStock
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_stock_schema_with_stable_field_names() {
        let stock = Stock {
            id: StockId {
                symbol: "0126Z0".to_string(),
                isin: None,
                reuters_code: Some("0126Z0".to_string()),
            },
            name: "삼성에피스홀딩스".to_string(),
            market: StockMarket::Kospi,
            kind: StockKind::CommonStock,
            industry: Some("기타 금융업".to_string()),
            sector: None,
            main_product: Some("지주회사".to_string()),
            listed_at: Some("2025-11-24".to_string()),
            fiscal_month: Some("12월".to_string()),
            homepage: Some("http://www.samsungepisholdings.com".to_string()),
            region: Some("인천광역시".to_string()),
            source: StockSource::Kind,
            source_url: Some("https://kind.krx.co.kr/corpgeneral/corpList.do".to_string()),
            updated_at: Some("2026-06-09".to_string()),
        };

        assert_eq!(
            serde_json::to_value(stock).unwrap(),
            json!({
                "id": {
                    "symbol": "0126Z0",
                    "isin": null,
                    "reuters_code": "0126Z0"
                },
                "name": "삼성에피스홀딩스",
                "market": "KOSPI",
                "kind": "common_stock",
                "industry": "기타 금융업",
                "sector": null,
                "main_product": "지주회사",
                "listed_at": "2025-11-24",
                "fiscal_month": "12월",
                "homepage": "http://www.samsungepisholdings.com",
                "region": "인천광역시",
                "source": "KIND",
                "source_url": "https://kind.krx.co.kr/corpgeneral/corpList.do",
                "updated_at": "2026-06-09"
            })
        );
    }

    #[test]
    fn accepts_alphanumeric_symbols() {
        assert!(validate_symbol("005930").is_ok());
        assert!(validate_symbol("0126Z0").is_ok());
        assert!(validate_symbol("0126-0").is_err());
    }

    #[test]
    fn normalizes_kind_market_names() {
        assert_eq!(normalize_kind_market("유가"), StockMarket::Kospi);
        assert_eq!(normalize_kind_market("코스닥"), StockMarket::Kosdaq);
        assert_eq!(normalize_kind_market("코넥스"), StockMarket::Konex);
    }

    #[test]
    fn infers_common_stock_types() {
        assert_eq!(
            infer_stock_kind("삼성전자", Some("통신 및 방송 장비 제조업"), None),
            StockKind::CommonStock
        );
        assert_eq!(
            infer_stock_kind("삼성전자우", None, None),
            StockKind::PreferredStock
        );
        assert_eq!(infer_stock_kind("KODEX 반도체", None, None), StockKind::Etf);
        assert_eq!(
            infer_stock_kind("하나 레버리지 반도체 ETN", None, None),
            StockKind::Etn
        );
    }
}
