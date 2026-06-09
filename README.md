# openstock

`openstock`은 증권사 API와 비증권 데이터 API를 CLI로 다루기 위한 Rust 프로그램입니다. 현재 증권사 API는 한국투자증권(KIS) 실전 API를 사용하고, 비증권 데이터는 KIND, Naver, OpenDART provider로 분리합니다.

이 프로그램은 투자 판단을 대신하지 않습니다. `order buy`, `order sell`은 실전 주문을 전송할 수 있으므로 실행 전 입력값과 계좌 상태를 반드시 확인해야 합니다.

## Architecture

| Layer | Path | Role |
| --- | --- | --- |
| Broker API | `src/apis` | 증권사별 구현체. 현재 `kis`가 실전 계좌 조회, 시세 조회, 주문을 담당합니다. |
| Provider API | `src/providers` | 증권사와 무관한 데이터 API. KIND universe, Naver search, OpenDART 공시를 담당합니다. |
| Core | `src/core` | 공통 trait, 출력 JSON, dotenv, HTTP agent, stock schema, cache policy를 담당합니다. |
| Commands | `src/commands` | CLI 명령과 출력 설명 필드를 구성합니다. |

## Output Contract

모든 정상 출력은 stdout에 JSON object 한 줄로 출력합니다.

```json
{
  "command": "실행 명령 이름",
  "description": "명령 결과 설명",
  "fields": [
    {
      "name": "필드명",
      "description": "AI와 사람이 해석할 수 있는 의미 설명",
      "value": "실제 값"
    }
  ],
  "raw": "원본 또는 상세 데이터"
}
```

오류는 stderr에 같은 설명형 JSON 구조로 출력합니다.

```json
{
  "command": "명령 이름",
  "description": "실패 설명",
  "fields": [
    {"name": "status", "description": "명령 실행 결과", "value": "error"},
    {"name": "message", "description": "오류 내용", "value": "오류 메시지"}
  ],
  "raw": null
}
```

## Environment Variables

`.env` 파일에서 읽고, 일부 값은 명령 실행 중 갱신됩니다.

| Name | Required For | Direction | Description |
| --- | --- | --- | --- |
| `KIS_APPKEY` | KIS login/call/account/market/order | read/write | 한국투자증권 Open API app key. `api login --appkey` 실행 시 저장될 수 있습니다. |
| `KIS_APPSECRET` | KIS login/call/account/market/order | read/write | 한국투자증권 Open API app secret. `api login --appsecret` 실행 시 저장될 수 있습니다. |
| `KIS_ACCESS_TOKEN` | KIS call/account/market/order | read/write | `api login`으로 발급 또는 재사용되는 접근 토큰입니다. |
| `KIS_ACCESS_TOKEN_EXPIRED_AT` | KIS login | read/write | 접근 토큰 만료 시각입니다. 유효하면 `api login`은 재발급하지 않습니다. |
| `KIS_TOKEN_TYPE` | KIS auth metadata | write | KIS token type입니다. 보통 `Bearer`입니다. |
| `KIS_ACCESS_TOKEN_EXPIRES_IN` | KIS auth metadata | write | 토큰 유효 초 단위 값입니다. |
| `KIS_ACCOUNT` | `account status`, `order buy`, `order sell`, `order status` | read | `CANO-ACNT_PRDT_CD` 형식 계좌입니다. 예: `12345678-01`. |
| `OPENDART_API_KEY` | `dart *` | read | OpenDART API 인증키입니다. |

`.env`는 `.gitignore` 대상입니다. 토큰과 계좌번호를 커밋하지 마십시오.

## Cache and File IO

로컬 캐시는 `.openstock` 아래에 생성되며 `.gitignore` 대상입니다.

| Path | Writer | Reader | Retention | Description |
| --- | --- | --- | --- | --- |
| `.openstock/universe/kind/latest.json` | `universe sync` | `universe status/list/chunks/validate` | protected | KIND 상장법인목록 최신 normalized stock list입니다. |
| `.openstock/universe/kind/meta.json` | `universe sync` | `universe status/*` | protected | universe cache metadata입니다. |
| `.openstock/universe/kind/YYYY-MM-DD.json` | `universe sync` | audit/manual use | max 7 files or 25MB | 날짜별 universe snapshot입니다. |
| `.openstock/opendart/corp_codes.json` | `dart sync`, `dart corp/disclosures/show` on cache miss | `dart corp/disclosures/show` | latest only | 상장사 `stock_code` to `corp_code` 매핑입니다. |
| `.openstock/opendart/corp_codes_meta.json` | `dart sync` | `dart status` | latest only | OpenDART corp code cache metadata입니다. |
| `.openstock/opendart/documents/{rcept_no}.zip` | `dart document`, `dart show` | `dart document`, `dart show` | max 100 files or 200MB | 공시서류 원본 ZIP cache입니다. |

캐시 상태와 정리는 다음 명령으로 수행합니다.

```bash
openstock cache status
openstock cache prune --dry-run
openstock cache prune
```

## Commands and Data IO

### `openstock version`

| Direction | Data |
| --- | --- |
| Input | 없음 |
| External IO | 없음 |
| File IO | 없음 |
| Output fields | `name`, `version` |
| Side effect | 없음 |

### `openstock api list`

등록된 증권사 API와 capability catalog를 반환합니다.

| Direction | Data |
| --- | --- |
| Input | 없음 |
| External IO | 없음 |
| File IO | 없음 |
| Output fields | `count`, `apis` |
| Raw | `null` |
| Side effect | 없음 |

### `openstock api login [--appkey KEY] [--appsecret SECRET] [--force]`

KIS 실전 API 접근 토큰을 발급하거나 유효한 기존 토큰을 재사용합니다.

| Direction | Data |
| --- | --- |
| Input | `--appkey`, `--appsecret`, `--force` |
| Env read | `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCESS_TOKEN`, `KIS_ACCESS_TOKEN_EXPIRED_AT` |
| Env write | `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCESS_TOKEN`, `KIS_ACCESS_TOKEN_EXPIRED_AT`, `KIS_TOKEN_TYPE`, `KIS_ACCESS_TOKEN_EXPIRES_IN` |
| External IO | `POST https://openapi.koreainvestment.com:9443/oauth2/tokenP` only when token is missing/expired or `--force` is used |
| Output fields | `broker`, `status`, `force`, `credential_source`, `token_storage`, `side_effect` |
| Side effect | writes auth state to `.env` |

### `openstock api call <endpoint> --param KEY=VALUE`

KIS endpoint를 직접 GET 호출합니다. `tr_id` 파라미터는 HTTP header로 전송하고 나머지는 query string으로 전송합니다.

| Direction | Data |
| --- | --- |
| Input | `endpoint`, repeated `--param KEY=VALUE` |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET` |
| External IO | KIS endpoint GET |
| File IO | `.env` read |
| Output fields | `broker`, `endpoint`, `params`, `request_semantics`, `response`, `response_semantics` |
| Raw | parsed KIS response |
| Side effect | endpoint에 따라 다름. typed command 사용 권장 |

### `openstock search <query>`

Naver 모바일 주식 검색 API로 종목 후보를 검색합니다.

| Direction | Data |
| --- | --- |
| Input | `query`: 종목명 또는 종목코드 |
| External IO | `GET https://m.stock.naver.com/front-api/search/autoComplete` |
| File IO | 없음 |
| Output fields | `provider`, `query`, `stocks` |
| Raw | Naver 원본 JSON |
| Side effect | 없음 |

`stocks` 항목은 가능한 경우 `code`, `name`, `market`, `market_code`, `nation_code`, `category`, `reuters_code`, `url`을 포함합니다.

### `openstock universe sync [--force]`

KIND 상장법인목록을 받아 stock universe cache를 갱신합니다.

| Direction | Data |
| --- | --- |
| Input | `--force` |
| External IO | `GET https://kind.krx.co.kr/corpgeneral/corpList.do?method=download&searchType=13` |
| File write | `.openstock/universe/kind/latest.json`, `meta.json`, `YYYY-MM-DD.json` |
| Output fields | `source`, `cache_date`, `refreshed_at`, `refreshed`, `stock_count`, `counts_by_market` |
| Raw | universe metadata |
| Side effect | writes/updates local cache; prunes old snapshots |

### `openstock universe status`

| Direction | Data |
| --- | --- |
| Input | 없음 |
| File read | `.openstock/universe/kind/latest.json`, `meta.json` |
| Output fields | `source`, `cache_date`, `refreshed_at`, `refreshed`, `stock_count`, `counts_by_market` |
| Side effect | 없음 |

### `openstock universe list [--market MARKET] [--kind KIND] [--offset N] [--limit N]`

로컬 universe 종목을 필터링해 반환합니다.

| Direction | Data |
| --- | --- |
| Input | `--market KOSPI|KOSDAQ|KONEX`, `--kind common_stock|preferred_stock|etf|etn|spac|reit`, `--offset`, `--limit` |
| File read | universe cache |
| Output fields | `source`, `cache_date`, `total_count`, `filtered_count`, `offset`, `limit`, `stocks` |
| Side effect | cache miss면 sync 시도 가능 |

### `openstock universe chunks [--market MARKET] [--kind KIND] [--size N]`

Universe를 시장/종류별 chunk로 나눠 순회 단위를 만듭니다.

| Direction | Data |
| --- | --- |
| Input | `--market KOSPI|KOSDAQ|KONEX`, `--kind common_stock|preferred_stock|etf|etn|spac|reit`, `--size` |
| File read | universe cache |
| Output fields | `source`, `cache_date`, `filtered_count`, `chunk_size`, `chunk_count`, `chunks` |
| Side effect | 없음 |

### `openstock universe validate`

로컬 universe가 기대 규모와 대표 종목을 만족하는지 검증합니다.

| Direction | Data |
| --- | --- |
| Input | 없음 |
| File read | universe cache |
| Output fields | `valid`, `stock_count`, `counts_by_market`, `checks`, `errors` |
| Side effect | 없음 |

### `openstock market <symbol>`

KIS 현재가와 종목/기업 기본 정보를 조회합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`: 6자리 국내 종목코드 |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET` |
| External IO | KIS `GET /uapi/domestic-stock/v1/quotations/inquire-price`, TR ID `FHKST01010100`; KIS `GET /uapi/domestic-stock/v1/quotations/search-stock-info`, TR ID `CTPF1002R` |
| Output fields | `broker`, `symbol`, `price`, `company` |
| Raw | KIS price/company 원본 응답 |
| Side effect | 없음 |

### `openstock market history <symbol> --from YYYYMMDD --to YYYYMMDD [--period D|W|M|Y] [--raw-price]`

KIS 기간별 OHLCV를 조회합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`, `--from`, `--to`, `--period`, `--raw-price` |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET` |
| External IO | `GET /uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice`, TR ID `FHKST03010100` |
| Output fields | `broker`, `symbol`, `period`, `date_range`, `adjusted`, `count`, `candles`, `summary` |
| Raw | normalized payload plus KIS raw response |
| Side effect | 없음 |

`candles`는 날짜 오름차순이며 각 항목은 `date`, `open`, `high`, `low`, `close`, `volume`, `trading_value`, `change`, `change_sign`, `change_rate`를 포함합니다. 숫자는 원본 정밀도 보존을 위해 문자열로 반환합니다.

### `openstock dart sync [--force]`

OpenDART 고유번호 ZIP/XML을 받아 상장사 `stock_code` to `corp_code` cache를 갱신합니다.

| Direction | Data |
| --- | --- |
| Input | `--force` |
| Env read | `OPENDART_API_KEY` |
| External IO | `GET https://opendart.fss.or.kr/api/corpCode.xml` |
| File write | `.openstock/opendart/corp_codes.json`, `corp_codes_meta.json` |
| Output fields | `source`, `cache_date`, `refreshed_at`, `refreshed`, `total_count`, `listed_count` |
| Side effect | writes/updates local cache |

### `openstock dart status`

| Direction | Data |
| --- | --- |
| Input | 없음 |
| File read | OpenDART corp code cache |
| Output fields | `source`, `cache_date`, `refreshed_at`, `refreshed`, `total_count`, `listed_count` |
| Side effect | 없음 |

### `openstock dart corp <symbol>`

KRX 종목코드를 OpenDART 고유번호로 변환합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`: 6자리 종목코드 |
| File read/write | corp code cache. cache miss 또는 날짜 만료 시 refresh 시도 |
| External IO | cache refresh 시 OpenDART corpCode API |
| Output fields | `stock_code`, `corp_code`, `corp_name`, `modify_date` |
| Side effect | cache refresh 가능 |

### `openstock dart disclosures [symbol] [--corp-code CODE] [--from YYYYMMDD] [--to YYYYMMDD] [--corp-cls Y|K|N|E] [--page-no N] [--page-count N]`

전체 또는 특정 종목의 공시목록을 조회합니다.

| Direction | Data |
| --- | --- |
| Input | optional `symbol`, optional `--corp-code`, date range, corp class, page options |
| Env read | `OPENDART_API_KEY` |
| File read/write | `symbol` 사용 시 corp code cache read; cache refresh 가능 |
| External IO | `GET https://opendart.fss.or.kr/api/list.json` |
| Output fields | `corp_code`, `from`, `to`, `page`, `resolved`, `disclosures` |
| Raw | OpenDART list response |
| Side effect | cache refresh 가능 |

`disclosures` 항목은 OpenDART가 반환하는 `corp_cls`, `corp_code`, `corp_name`, `stock_code`, `rcept_no`, `report_nm`, `rcept_dt`, `flr_nm`, `rm` 등을 포함합니다.

### `openstock dart document <rcept_no> [--force] [--max-chars N]`

접수번호로 공시서류 원본 ZIP을 다운로드하고 XML 본문 텍스트를 추출합니다.

| Direction | Data |
| --- | --- |
| Input | `rcept_no`: 14자리 접수번호, `--force`, `--max-chars` |
| Env read | `OPENDART_API_KEY` |
| External IO | `GET https://opendart.fss.or.kr/api/document.xml` |
| File read/write | `.openstock/opendart/documents/{rcept_no}.zip` |
| Output fields | `rcept_no`, `source`, `viewer_url`, `cached`, `zip_path`, `zip_bytes`, `files`, `text`, `text_chars`, `truncated` |
| Raw | same document object |
| Side effect | writes ZIP cache; prunes old document ZIP files |

`text`는 XML tag, style, script를 제거한 본문입니다. `--max-chars`를 넘으면 앞부분만 출력하고 `truncated=true`를 반환합니다.

### `openstock dart show <symbol> [--from YYYYMMDD] [--to YYYYMMDD] [--index N] [--page-count N] [--force] [--max-chars N]`

종목코드 기준으로 공시목록을 조회하고 선택한 공시 원문까지 함께 반환합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`, optional date range, `--index`, `--page-count`, `--force`, `--max-chars` |
| Env read | `OPENDART_API_KEY` |
| File read/write | corp code cache, document ZIP cache |
| External IO | OpenDART `list.json`; selected document cache miss or `--force`이면 `document.xml` |
| Output fields | `symbol`, `resolved`, `date_range`, `selected_index`, `selected_disclosure`, `disclosures`, `document` |
| Raw | query, resolved corp, selected disclosure, list response, document object |
| Side effect | cache refresh/write 가능 |

### `openstock account status`

KIS 계좌 잔고와 보유종목을 조회합니다.

| Direction | Data |
| --- | --- |
| Input | 없음 |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS `GET /uapi/domestic-stock/v1/trading/inquire-balance` |
| Output fields | `broker`, `account`, `balance`, `holdings` |
| Raw | KIS balance response |
| Side effect | 없음 |

### `openstock order buy <symbol> --qty QTY (--price PRICE|--market)`

국내주식 매수 주문을 전송합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`, `--qty`, one of `--price` or `--market` |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS `POST /uapi/domestic-stock/v1/trading/order-cash`, TR ID `TTTC0802U` |
| Output fields | `broker`, `side`, `symbol`, `qty`, `order_type`, `price`, `order` |
| Raw | KIS order response |
| Side effect | financial_order: 실전 매수 주문 |

### `openstock order sell <symbol> --qty QTY (--price PRICE|--market)`

국내주식 매도 주문을 전송합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`, `--qty`, one of `--price` or `--market` |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS `POST /uapi/domestic-stock/v1/trading/order-cash`, TR ID `TTTC0801U` |
| Output fields | `broker`, `side`, `symbol`, `qty`, `order_type`, `price`, `order` |
| Raw | KIS order response |
| Side effect | financial_order: 실전 매도 주문 |

### `openstock order status [order_no] [--from YYYYMMDD] [--to YYYYMMDD]`

주문 및 체결 내역을 조회합니다.

| Direction | Data |
| --- | --- |
| Input | optional `order_no`, optional date range |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS `GET /uapi/domestic-stock/v1/trading/inquire-daily-ccld` |
| Output fields | `broker`, `account`, `order_no`, `orders`, `summary` |
| Raw | KIS order status response |
| Side effect | 없음 |

### `openstock cache status`

| Direction | Data |
| --- | --- |
| Input | 없음 |
| File read | `.openstock` recursively |
| Output fields | `root`, `exists`, `total_files`, `total_bytes`, `namespaces` |
| Side effect | 없음 |

### `openstock cache prune [--dry-run]`

캐시 보존 정책을 적용합니다.

| Direction | Data |
| --- | --- |
| Input | `--dry-run` |
| File read | `.openstock` cache directories |
| File delete | old universe snapshots, old OpenDART document ZIPs unless `--dry-run` |
| Output fields | `dry_run`, `deleted_files`, `deleted_bytes`, `reports` |
| Side effect | deletes local cache files unless dry-run |

## Stock Schema

Universe 종목은 다음 구조로 정규화됩니다.

| Field | Type | Description |
| --- | --- | --- |
| `id.symbol` | string | KRX 종목코드. 알파벳 포함 코드를 허용합니다. |
| `id.isin` | string/null | ISIN. 현재 KIND source에서는 null일 수 있습니다. |
| `id.reuters_code` | string/null | Reuters style code. |
| `name` | string | 종목명 |
| `market` | enum | `KOSPI`, `KOSDAQ`, `KONEX`, `OTHER`, `UNKNOWN` |
| `kind` | enum | `common_stock`, `preferred_stock`, `etf`, `etn`, `spac`, `reit`, `other`, `unknown` |
| `industry` | string/null | 업종 |
| `sector` | string/null | 섹터. 현재 null 가능 |
| `main_product` | string/null | 주요 제품 |
| `listed_at` | string/null | 상장일 |
| `fiscal_month` | string/null | 결산월 |
| `homepage` | string/null | 홈페이지 |
| `region` | string/null | 지역 |
| `source` | enum | 현재 `KIND` |
| `source_url` | string/null | 원천 URL |
| `updated_at` | string/null | 캐시 기준일 |

## External Data Sources

| Source | API | Used By |
| --- | --- | --- |
| KIS | `https://openapi.koreainvestment.com:9443` | login, account, market, market history, order |
| KIND | `https://kind.krx.co.kr/corpgeneral/corpList.do?method=download&searchType=13` | universe |
| Naver | `https://m.stock.naver.com/front-api/search/autoComplete` | search |
| OpenDART | `https://opendart.fss.or.kr/api/corpCode.xml` | dart sync/corp/show/disclosures |
| OpenDART | `https://opendart.fss.or.kr/api/list.json` | dart disclosures/show |
| OpenDART | `https://opendart.fss.or.kr/api/document.xml` | dart document/show |

## Typical Evidence Workflow

```bash
openstock api login
openstock universe sync
openstock dart sync
openstock search 삼성전자
openstock market 005930
openstock market history 005930 --from 20260101 --to 20260609
openstock dart show 005930 --from 20260601 --to 20260609 --index 1
openstock account status
```

At this point the program can collect evidence for a trade decision. Automated candidate generation, risk checking, and backtesting should be implemented before unattended trading.
