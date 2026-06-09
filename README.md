# openstock

`openstock`은 증권사 API와 비증권 데이터 API를 CLI로 다루기 위한 Rust 프로그램입니다. 현재 증권사 API는 한국투자증권(KIS) 실전 API를 사용하고, 비증권 데이터는 KIND, Naver, OpenDART provider로 분리합니다.

이 프로그램은 투자 판단을 대신하지 않습니다. `order buy`, `order sell`은 실전 주문을 전송할 수 있으므로 실행 전 입력값과 계좌 상태를 반드시 확인해야 합니다.

## Install

로컬 repo에서 release 바이너리를 빌드해 `~/.local/bin/openstock`에 설치합니다.

```bash
./scripts/install.sh
```

원라인 설치:

```bash
curl -fsSL https://git.hananakick.cc/Autotrade/openstock/raw/branch/main/scripts/install.sh | sh
```

원라인 설치 스크립트는 현재 디렉터리에 openstock source tree가 있으면 그대로 빌드하고, 없으면 main branch archive를 임시 디렉터리에 내려받아 빌드합니다. 설치 위치는 `OPENSTOCK_INSTALL_DIR`, 바이너리 이름은 `OPENSTOCK_BIN_NAME`, archive URL은 `OPENSTOCK_ARCHIVE_URL`로 바꿀 수 있습니다.

```bash
OPENSTOCK_INSTALL_DIR=/usr/local/bin ./scripts/install.sh
```

삭제:

```bash
./scripts/uninstall.sh
```

원라인 삭제:

```bash
curl -fsSL https://git.hananakick.cc/Autotrade/openstock/raw/branch/main/scripts/uninstall.sh | sh
```

삭제 스크립트는 설치된 바이너리만 제거하고 `~/.config/openstock`의 설정/캐시는 보존합니다.

업데이트:

```bash
openstock update
```

원라인 업데이트:

```bash
curl -fsSL https://git.hananakick.cc/Autotrade/openstock/raw/branch/main/scripts/update.sh | sh
```

`openstock update`는 Gitea 최신 release를 조회하고, 현재 버전보다 최신이면 Linux x86_64 release asset을 내려받아 현재 실행 중인 바이너리의 디렉터리에 설치합니다. 설치 위치를 직접 지정하려면 `OPENSTOCK_INSTALL_DIR=/path openstock update`를 사용합니다. 같은 버전도 재설치하려면 `openstock update --force`를 사용합니다.

## Release

Gitea Actions가 활성화된 저장소에서는 push 시 자동으로 CI가 실행됩니다. `main` branch push는 테스트와 release 빌드 검증만 수행하고, `v*` tag push는 Gitea Release를 만들고 Linux x86_64 바이너리를 asset으로 등록합니다.

```bash
git tag v0.2.1
git push origin main v0.2.1
```

필요 조건:

| Requirement | Meaning |
| --- | --- |
| Gitea Actions | instance와 repository에서 Actions가 활성화되어 있어야 합니다. |
| act runner | `ubuntu-latest` job을 실행할 전역 또는 repository runner가 등록되어 있어야 합니다. |
| release permission | workflow의 `${{ secrets.GITEA_TOKEN }}`가 release 생성/asset 업로드 권한을 가져야 합니다. |

### Runner

Docker 기반 Gitea act runner 구성은 `ops/gitea-runner`에 있습니다. 기본 운영 방식은 전역(instance) runner입니다. 등록 토큰은 Gitea admin settings의 Actions runner 화면에서 발급받아 로컬 `.env`에만 저장합니다.

```bash
cp ops/gitea-runner/.env.example ops/gitea-runner/.env
vi ops/gitea-runner/.env
./scripts/runner-up.sh
```

`GITEA_INSTANCE_URL`은 SSH remote host가 아니라 브라우저에서 Gitea UI가 실제로 열리는 HTTP(S) root URL이어야 합니다. 예를 들어 `ssh://git@git.example.com:2222/org/repo.git`로 push하더라도 Gitea 웹이 `https://code.example.com`에서 열린다면 `GITEA_INSTANCE_URL=https://code.example.com`을 사용합니다. `./scripts/runner-up.sh`는 시작 전에 `${GITEA_INSTANCE_URL}/api/v1/version` 접근을 검사합니다.

러너 상태와 로그:

```bash
./scripts/runner-logs.sh
```

중지:

```bash
./scripts/runner-down.sh
```

runner는 `/var/run/docker.sock`을 mount하므로 이 머신의 Docker 권한을 가진 신뢰 가능한 repository에서만 사용해야 합니다. 전역 runner는 여러 repository가 공유할 수 있으므로 Gitea instance에서 runner 사용 범위를 신뢰 가능한 repository로 제한해 운영합니다. `ops/gitea-runner/.env`와 `ops/gitea-runner/data/`는 registration token과 runner state를 포함할 수 있어 git에서 제외합니다.

`Cannot ping the Gitea instance server`와 `permission_denied: 403 Forbidden`이 반복되면 runner 컨테이너는 실행됐지만 Gitea가 등록을 거부한 상태입니다. 다음 순서로 처리합니다.

1. `GITEA_INSTANCE_URL`이 Gitea API를 가리키는지 확인합니다. `${GITEA_INSTANCE_URL}/api/v1/version`이 Gitea JSON을 반환해야 합니다.
2. Gitea instance와 repository에서 Actions가 활성화되어 있는지 확인합니다.
3. admin settings에서 instance runner registration token을 새로 발급합니다.
4. `ops/gitea-runner/.env`의 `GITEA_RUNNER_REGISTRATION_TOKEN` 값을 새 token으로 교체합니다.
5. runner state를 초기화하고 재시작합니다.

```bash
cd ops/gitea-runner
docker compose down
rm -rf data
docker compose up -d
docker compose logs --tail=120 openstock-runner
```

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
  "raw": "직접 API 호출 원본 또는 null"
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

## Output Field Reference

모든 명령의 top-level JSON은 다음 공통 키를 가집니다.

| Key | Type | Meaning |
| --- | --- | --- |
| `command` | string | 실행한 CLI 명령의 논리 이름입니다. 예: `market history`, `dart show`. |
| `description` | string | 전체 결과가 무엇을 의미하는지 설명합니다. |
| `fields` | array | 사람이 읽기 쉬운 필드 목록입니다. 각 항목은 `name`, `description`, `value`를 가집니다. |
| `fields[].name` | string | 출력 필드 이름입니다. |
| `fields[].description` | string | 해당 필드의 의미입니다. CLI에서 key 의미를 해석할 때 이 값을 우선 사용합니다. |
| `fields[].value` | any | 실제 값입니다. 원본 API 숫자 문자열은 정밀도 보존을 위해 문자열일 수 있습니다. |
| `raw` | any/null | 중복 출력을 줄이기 위해 typed command는 기본적으로 `null`입니다. `api call`처럼 원본 응답 자체가 목적인 명령만 원본 응답을 넣습니다. |

오류 출력의 `fields`는 다음 고정 키를 사용합니다.

| Field | Type | Meaning |
| --- | --- | --- |
| `status` | string | 항상 `error`입니다. |
| `message` | string | 실패 원인 메시지입니다. |

## Agent CLI Skill

에이전트는 별도 MCP 서버나 plugin wrapper 없이 `openstock` CLI를 직접 실행하는 전략을 사용합니다.

| Document | Purpose |
| --- | --- |
| `skills/openstock-agent-cli/SKILL.md` | 에이전트가 따라야 할 실행 전략, 출력 해석 규칙, 주문 안전 규칙 |
| `skills/openstock-agent-cli/references/commands/` | CLI depth별 명령 설명과 IO 계약 |

### Command Output Fields

#### `version`

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | string | 프로그램 이름입니다. Cargo package name과 같습니다. |
| `version` | string | 현재 실행 중인 openstock 버전입니다. |

#### `update`

| Field | Type | Meaning |
| --- | --- | --- |
| `release_api_url` | string | 최신 release 정보를 조회한 Gitea API URL입니다. |
| `current_version` | string | 현재 실행 중인 openstock 버전입니다. |
| `latest_version` | string | Gitea 최신 release tag에서 해석한 버전입니다. |
| `release_tag` | string | Gitea 최신 release tag입니다. |
| `release_url` | string | Gitea release 페이지 URL입니다. |
| `asset_name` | string | 설치에 사용한 release asset 이름입니다. |
| `asset_url` | string | 설치에 사용한 release asset 다운로드 URL입니다. |
| `install_dir` | string | 업데이트 대상 바이너리를 설치한 디렉터리입니다. |
| `status` | string | 업데이트 명령 실행 결과입니다. `updated` 또는 `up_to_date`입니다. |
| `stdout` | string | release asset 설치 스크립트의 표준 출력입니다. |
| `stderr` | string | release asset 설치 스크립트의 표준 오류 출력입니다. |

#### `api list`

| Field | Type | Meaning |
| --- | --- | --- |
| `count` | number | 등록된 증권사 API 구현체 개수입니다. |
| `apis` | array | 증권사별 metadata, 인증 요구사항, 지원 명령, 입출력 계약, 부작용 정보를 담은 catalog입니다. |

`apis[]` 주요 구조:

| Key | Type | Meaning |
| --- | --- | --- |
| `id` | string | 증권사 API 식별자입니다. 예: `KIS`. |
| `name` | string | 증권사 표시명입니다. |
| `description` | string | 증권사 API 설명입니다. |
| `credential_requirements` | array | 필요한 환경변수와 사용처입니다. |
| `capabilities` | array | CLI 명령별 목적, 입력, 출력 계약, 부작용 목록입니다. |

#### `api login`

| Field | Type | Meaning |
| --- | --- | --- |
| `broker` | string | 로그인 대상 증권사 API입니다. 현재 `KIS`입니다. |
| `status` | string | 로그인 처리 결과입니다. 성공 시 `success`입니다. |
| `force` | boolean | 기존 유효 토큰을 무시하고 새 토큰 발급을 시도했는지 여부입니다. |
| `credential_source` | object | `appkey`, `appsecret`을 CLI 인자에서 읽었는지 `.env`에서 읽었는지 나타냅니다. |
| `token_storage` | object | 토큰과 만료시각이 저장되는 파일과 env key입니다. |
| `side_effect` | string | 명령 부작용입니다. `writes_auth_state`이면 `.env` 인증 상태를 쓸 수 있습니다. |

#### `api call`

| Field | Type | Meaning |
| --- | --- | --- |
| `broker` | string | 호출에 사용한 증권사 API입니다. |
| `endpoint` | string | 호출한 KIS API 경로 또는 전체 URL입니다. |
| `params` | array | 요청 파라미터 목록입니다. 각 항목은 `name`, `value`, `transport`, `description`을 가집니다. |
| `request_semantics` | object | HTTP method, 인증 방식, `tr_id` 처리 방식, 부작용 위험을 설명합니다. |
| `response` | object/array/string | 파싱된 API 응답 값입니다. |
| `response_semantics` | array | 응답 top-level key의 의미와 값 타입 설명입니다. |

#### `search`

| Field | Type | Meaning |
| --- | --- | --- |
| `provider` | string | 검색 데이터 제공자입니다. 현재 `NAVER`입니다. |
| `query` | string | 사용자가 입력한 검색어입니다. |
| `stocks` | array | 검색된 종목 후보 목록입니다. |

`stocks[]` 주요 구조:

| Key | Type | Meaning |
| --- | --- | --- |
| `code` | string | 종목코드입니다. 국내 주식이면 보통 6자리 KRX 코드입니다. |
| `name` | string | 종목명입니다. |
| `market` | string | Naver가 반환한 시장 표시명입니다. |
| `market_code` | string | Naver가 반환한 시장 코드입니다. 없으면 빈 문자열일 수 있습니다. |
| `nation_code` | string | 국가 코드입니다. 없으면 빈 문자열일 수 있습니다. |
| `category` | string | 종목 카테고리입니다. 없으면 빈 문자열일 수 있습니다. |
| `reuters_code` | string | Reuters style code입니다. 없으면 빈 문자열일 수 있습니다. |
| `url` | string | Naver 모바일 종목 페이지 URL입니다. |

#### `score set`, `score get`, `score list`, `score delete`

| Field | Type | Meaning |
| --- | --- | --- |
| `path` | string | 종목 평가 점수를 저장하는 파일 경로입니다. 기본값은 `~/.config/openstock/scores.json`입니다. |
| `symbol` | string | 점수를 매긴 종목코드 또는 종목 ID입니다. 영문자는 대문자로 정규화됩니다. |
| `score` | number | 종목 평가 점수입니다. 0은 최저, 100은 최고입니다. |
| `updated_at_unix` | number | 점수를 저장하거나 갱신한 Unix timestamp(초)입니다. |
| `count` | number | `score list`가 반환한 저장 점수 개수입니다. |
| `scores` | array | `score list`가 반환한 점수 목록입니다. 점수 내림차순, 종목 ID 오름차순으로 정렬됩니다. |
| `deleted` | boolean | `score delete`에서 저장된 점수가 실제로 삭제되었는지 여부입니다. |
| `removed` | object/null | `score delete`에서 삭제된 기존 점수 기록입니다. |

#### `universe sync`, `universe status`

| Field | Type | Meaning |
| --- | --- | --- |
| `source` | string | universe 원천입니다. 현재 `KIND`입니다. |
| `source_url` | string | 원천 데이터 다운로드 URL입니다. |
| `cache_date` | string | 캐시 기준일입니다. `YYYY-MM-DD` 형식입니다. |
| `refreshed_at` | string | 캐시 갱신 시각입니다. UTC ISO-like 문자열입니다. |
| `refreshed` | boolean | 이번 명령에서 새로 다운로드했는지 여부입니다. |
| `stock_count` | number | 캐시에 저장된 전체 종목 수입니다. |
| `counts_by_market` | object | 시장별 종목 수입니다. 예: `KOSPI`, `KOSDAQ`, `KONEX`. |

#### `universe list`

| Field | Type | Meaning |
| --- | --- | --- |
| `source` | string | universe 원천입니다. |
| `cache_date` | string | 사용한 universe 캐시 기준일입니다. |
| `total_count` | number | 필터 적용 전 전체 종목 수입니다. |
| `filtered_count` | number | `--market`, `--kind` 필터 적용 후 종목 수입니다. |
| `offset` | number | 반환 시작 위치입니다. |
| `limit` | number | 최대 반환 종목 수입니다. |
| `stocks` | array | 반환된 종목 목록입니다. 구조는 [Stock Schema](#stock-schema)를 따릅니다. |

#### `universe chunks`

| Field | Type | Meaning |
| --- | --- | --- |
| `source` | string | universe 원천입니다. |
| `cache_date` | string | 사용한 universe 캐시 기준일입니다. |
| `filtered_count` | number | chunk 생성 대상 종목 수입니다. |
| `chunk_size` | number | chunk당 최대 종목 수입니다. |
| `chunk_count` | number | 생성된 chunk 개수입니다. |
| `chunks` | array | scan 가능한 chunk 목록입니다. |

`chunks[]` 구조:

| Key | Type | Meaning |
| --- | --- | --- |
| `chunk_id` | string | chunk 식별자입니다. 시장, 종목 유형, index를 포함합니다. |
| `index` | number | 같은 시장/종류 그룹 안의 1부터 시작하는 chunk 번호입니다. |
| `size` | number | chunk 생성 기준 크기입니다. |
| `count` | number | 이 chunk에 포함된 실제 종목 수입니다. |
| `start_symbol` | string/null | chunk 첫 종목코드입니다. |
| `end_symbol` | string/null | chunk 마지막 종목코드입니다. |
| `market` | string | chunk 시장입니다. |
| `kind` | string | chunk 종목 유형입니다. |

#### `universe validate`

| Field | Type | Meaning |
| --- | --- | --- |
| `valid` | boolean | 전체 검증 통과 여부입니다. |
| `stock_count` | number | 검증 대상 전체 종목 수입니다. |
| `counts_by_market` | object | 시장별 종목 수입니다. |
| `checks` | array | 개별 검증 항목 결과입니다. |
| `errors` | array | 실패한 검증 항목 설명입니다. |

`checks[]` 구조:

| Key | Type | Meaning |
| --- | --- | --- |
| `name` | string | 검증 항목 이름입니다. |
| `description` | string | 검증 항목 의미입니다. |
| `valid` | boolean | 해당 항목 통과 여부입니다. |
| `expected` | any | 기대 조건입니다. |
| `actual` | any | 실제 값입니다. |

#### `market`

| Field | Type | Meaning |
| --- | --- | --- |
| `broker` | string | 조회에 사용한 증권사 API입니다. |
| `symbol` | string | 조회한 종목코드입니다. |
| `price` | object | KIS 현재가/호가/시세/투자주의 관련 값입니다. |
| `company` | object | KIS 종목 및 기업 기본 정보입니다. |

`price`와 `company`는 KIS field name을 보존한 조회 결과입니다.

#### `market history`

| Field | Type | Meaning |
| --- | --- | --- |
| `broker` | string | 조회에 사용한 증권사 API입니다. |
| `symbol` | string | 조회한 종목코드입니다. |
| `period` | string | 봉 단위입니다. `D`, `W`, `M`, `Y` 중 하나입니다. |
| `date_range` | object | 요청 기간입니다. `from`, `to`를 포함합니다. |
| `adjusted` | boolean | 수정주가 조회 여부입니다. `false`이면 원주가 조회입니다. |
| `count` | number | 조회된 가격 봉 개수입니다. |
| `candles` | array | 날짜 오름차순 OHLCV 배열입니다. |
| `summary` | object | KIS가 반환한 종목 요약 정보입니다. |

`candles[]` 구조:

| Key | Type | Meaning |
| --- | --- | --- |
| `date` | string | 거래일입니다. `YYYYMMDD` 형식입니다. |
| `open` | string | 시가입니다. |
| `high` | string | 고가입니다. |
| `low` | string | 저가입니다. |
| `close` | string | 종가입니다. |
| `volume` | string | 누적 거래량입니다. |
| `trading_value` | string | 누적 거래대금입니다. |
| `change` | string | 전일 대비 가격 변화입니다. |
| `change_sign` | string | KIS 전일 대비 부호 코드입니다. |
| `change_rate` | string | 전일 대비율입니다. KIS 응답에 없으면 빈 문자열일 수 있습니다. |

#### `dart sync`, `dart status`

| Field | Type | Meaning |
| --- | --- | --- |
| `source` | string | 공시정보 원천입니다. 현재 `OpenDART`입니다. |
| `cache_date` | string | 캐시 기준일입니다. |
| `refreshed_at` | string | 캐시 갱신 시각입니다. |
| `refreshed` | boolean | 이번 명령에서 새로 다운로드했는지 여부입니다. |
| `total_count` | number | OpenDART 고유번호 전체 항목 수입니다. |
| `listed_count` | number | `stock_code`가 존재하는 상장사 매핑 수입니다. |

#### `dart corp`

| Field | Type | Meaning |
| --- | --- | --- |
| `stock_code` | string | KRX 종목코드입니다. |
| `corp_code` | string | OpenDART 공시 API에서 사용하는 8자리 고유번호입니다. |
| `corp_name` | string | OpenDART에 등록된 회사명입니다. |
| `modify_date` | string | OpenDART 고유번호 정보의 최근 변경일자입니다. |

#### `dart disclosures`

| Field | Type | Meaning |
| --- | --- | --- |
| `corp_code` | string/null | 조회에 사용한 OpenDART 고유번호입니다. 없으면 전체 공시 조회입니다. |
| `from` | string/null | 검색 시작일입니다. `YYYYMMDD` 형식입니다. |
| `to` | string/null | 검색 종료일입니다. `YYYYMMDD` 형식입니다. |
| `page` | object | 페이지 정보입니다. `page_no`, `page_count`, `total_count`, `total_page`를 포함합니다. |
| `resolved` | object | 종목코드를 입력한 경우 DART 고유번호로 변환한 정보입니다. |
| `disclosures` | array | 조회된 공시 목록입니다. |

`disclosures[]` 주요 구조:

| Key | Type | Meaning |
| --- | --- | --- |
| `corp_cls` | string | 법인구분입니다. `Y` 유가, `K` 코스닥, `N` 코넥스, `E` 기타입니다. |
| `corp_code` | string | OpenDART 고유번호입니다. |
| `corp_name` | string | 회사명입니다. |
| `stock_code` | string | 종목코드입니다. |
| `rcept_no` | string | 공시 접수번호입니다. `dart document` 입력값입니다. |
| `report_nm` | string | 공시 보고서명입니다. |
| `rcept_dt` | string | 공시 접수일입니다. `YYYYMMDD` 형식입니다. |
| `flr_nm` | string | 제출인명입니다. |
| `rm` | string | OpenDART 비고 코드입니다. |

#### `dart document`

| Field | Type | Meaning |
| --- | --- | --- |
| `rcept_no` | string | DART 공시 접수번호입니다. |
| `source` | string | 공시서류 원본파일 제공 원천입니다. |
| `viewer_url` | string | 브라우저에서 확인 가능한 DART 공시 뷰어 URL입니다. |
| `cached` | boolean | 로컬 ZIP 캐시를 재사용했는지 여부입니다. |
| `zip_path` | string | 로컬에 저장된 공시서류 ZIP 캐시 경로입니다. |
| `zip_bytes` | number | ZIP 파일 크기입니다. |
| `files` | array | ZIP 내부 파일 목록과 추출 텍스트 길이입니다. |
| `text` | string | XML tag/style/script를 제거한 공시 본문 텍스트입니다. |
| `text_chars` | number | 출력된 본문 텍스트 글자 수입니다. |
| `truncated` | boolean | `--max-chars` 제한으로 본문이 잘렸는지 여부입니다. |

`files[]` 구조:

| Key | Type | Meaning |
| --- | --- | --- |
| `name` | string | ZIP 내부 파일명입니다. |
| `bytes` | number | ZIP 내부 원본 파일 크기입니다. |
| `text_chars` | number | 해당 파일에서 추출한 텍스트 글자 수입니다. |

#### `dart show`

| Field | Type | Meaning |
| --- | --- | --- |
| `symbol` | string | 조회한 KRX 종목코드입니다. |
| `resolved` | object | 종목코드를 OpenDART 고유번호로 변환한 정보입니다. |
| `date_range` | object | 공시목록 검색 기간입니다. null이면 OpenDART 기본 검색 범위입니다. |
| `selected_index` | number | 원문을 조회한 공시목록 항목 번호입니다. 1부터 시작합니다. |
| `selected_disclosure` | object | 원문 조회 대상으로 선택된 공시 항목입니다. |
| `disclosures` | array | 조회된 공시목록입니다. 다른 항목은 `--index`로 선택합니다. |
| `document` | object | 선택된 공시의 원문 ZIP 캐시 정보와 추출 본문입니다. 구조는 `dart document`와 같습니다. |

#### `account status`

| Field | Type | Meaning |
| --- | --- | --- |
| `broker` | string | 조회에 사용한 증권사 API입니다. |
| `account` | string | 조회한 계좌번호입니다. |
| `balance` | array | KIS 잔고/평가금/예수금 요약 목록입니다. |
| `holdings` | array | 보유종목 상세 목록입니다. |

`balance`와 `holdings`는 KIS field name을 보존한 조회 결과입니다.

#### `order buy`, `order sell`

| Field | Type | Meaning |
| --- | --- | --- |
| `broker` | string | 주문에 사용한 증권사 API입니다. |
| `side` | string | 주문 방향입니다. `buy` 또는 `sell`입니다. |
| `symbol` | string | 주문한 종목코드입니다. |
| `qty` | number | 주문 수량입니다. |
| `order_type` | string | 주문 유형입니다. `limit` 또는 `market`입니다. |
| `price` | number/null | 지정가 주문 가격입니다. 시장가 주문이면 null입니다. |
| `order` | object | 증권사에서 반환한 주문 접수 정보입니다. |

#### `order status`

| Field | Type | Meaning |
| --- | --- | --- |
| `broker` | string | 조회에 사용한 증권사 API입니다. |
| `account` | string | 조회한 계좌번호입니다. |
| `order_no` | string/null | 조회 대상으로 지정한 주문번호입니다. 없으면 기간 내 전체 주문입니다. |
| `orders` | array | 주문 및 체결 상세 목록입니다. |
| `summary` | object | 주문 조회 요약 정보입니다. |

#### `cache status`

| Field | Type | Meaning |
| --- | --- | --- |
| `root` | string | 캐시 루트 디렉터리입니다. |
| `exists` | boolean | 캐시 루트 디렉터리 존재 여부입니다. |
| `total_files` | number | 캐시 파일 총 개수입니다. |
| `total_bytes` | number | 캐시 총 용량입니다. bytes 단위입니다. |
| `namespaces` | array | 최상위 namespace별 파일 수와 용량입니다. |

`namespaces[]` 구조:

| Key | Type | Meaning |
| --- | --- | --- |
| `namespace` | string | cache root 아래 최상위 디렉터리 이름입니다. 예: `universe`, `opendart`. |
| `files` | number | namespace 안의 파일 수입니다. |
| `bytes` | number | namespace 안의 총 파일 크기입니다. |

#### `cache prune`

| Field | Type | Meaning |
| --- | --- | --- |
| `dry_run` | boolean | 실제 삭제 없이 삭제 예정 항목만 계산했는지 여부입니다. |
| `deleted_files` | number | 삭제했거나 dry-run에서 삭제 예정인 파일 수입니다. |
| `deleted_bytes` | number | 삭제했거나 dry-run에서 삭제 예정인 용량입니다. bytes 단위입니다. |
| `reports` | array | namespace별 보존 정책과 정리 결과입니다. |

`reports[]` 구조:

| Key | Type | Meaning |
| --- | --- | --- |
| `policy` | object | 적용한 보존 정책입니다. namespace, directory, max file/byte limit, protected files를 포함합니다. |
| `dry_run` | boolean | 이 report가 dry-run인지 여부입니다. |
| `deleted_files` | number | 해당 namespace에서 삭제했거나 삭제 예정인 파일 수입니다. |
| `deleted_bytes` | number | 해당 namespace에서 삭제했거나 삭제 예정인 용량입니다. |
| `retained_snapshot_files` | number | 보존된 snapshot 파일 수입니다. |
| `retained_snapshot_bytes` | number | 보존된 snapshot 파일 총 용량입니다. |
| `deleted_paths` | array | 삭제했거나 삭제 예정인 파일 경로 목록입니다. |

## Environment Variables

설정 파일은 기본적으로 `~/.config/openstock/.env`에서 읽고, 일부 값은 명령 실행 중 갱신됩니다. `OPENSTOCK_CONFIG_DIR`를 설정하면 이 루트 디렉터리를 바꿀 수 있습니다. 예를 들어 `OPENSTOCK_CONFIG_DIR=/tmp/openstock-test`이면 env 파일은 `/tmp/openstock-test/.env`, 캐시는 `/tmp/openstock-test/cache` 아래에 생성됩니다.

바이너리로 빌드해 어느 디렉터리에서 실행하더라도 기본 설정/캐시 위치는 동일합니다.

```bash
mkdir -p ~/.config/openstock
cp .env.example ~/.config/openstock/.env
```

이전 개발 버전에서 프로젝트 루트의 `.env`와 `.openstock`을 사용했다면 다음처럼 옮길 수 있습니다.

```bash
mkdir -p ~/.config/openstock
cp .env ~/.config/openstock/.env
mkdir -p ~/.config/openstock/cache
cp -R .openstock/* ~/.config/openstock/cache/
```

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

프로젝트 루트의 `.env`는 더 이상 런타임 기본 경로가 아닙니다. 토큰과 계좌번호를 커밋하지 마십시오.

## Cache and File IO

로컬 캐시는 기본적으로 `~/.config/openstock/cache` 아래에 생성됩니다. `OPENSTOCK_CONFIG_DIR`를 사용하면 `{OPENSTOCK_CONFIG_DIR}/cache` 아래에 생성됩니다.

| Path | Writer | Reader | Retention | Description |
| --- | --- | --- | --- | --- |
| `~/.config/openstock/scores.json` | `score set`, `score delete` | `score get`, `score list` | protected | 종목 ID별 0~100 평가 점수와 갱신시각입니다. `.env`와 같은 설정 디렉터리에 저장됩니다. |
| `~/.config/openstock/cache/universe/kind/latest.json` | `universe sync` | `universe status/list/chunks/validate` | protected | KIND 상장법인목록 최신 normalized stock list입니다. |
| `~/.config/openstock/cache/universe/kind/meta.json` | `universe sync` | `universe status/*` | protected | universe cache metadata입니다. |
| `~/.config/openstock/cache/universe/kind/YYYY-MM-DD.json` | `universe sync` | audit/manual use | max 7 files or 25MB | 날짜별 universe snapshot입니다. |
| `~/.config/openstock/cache/opendart/corp_codes.json` | `dart sync`, `dart corp/disclosures/show` on cache miss | `dart corp/disclosures/show` | latest only | 상장사 `stock_code` to `corp_code` 매핑입니다. |
| `~/.config/openstock/cache/opendart/corp_codes_meta.json` | `dart sync` | `dart status` | latest only | OpenDART corp code cache metadata입니다. |
| `~/.config/openstock/cache/opendart/documents/{rcept_no}.zip` | `dart document`, `dart show` | `dart document`, `dart show` | max 100 files or 200MB | 공시서류 원본 ZIP cache입니다. |

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

### `openstock update`

Gitea 최신 release asset을 내려받아 현재 openstock 바이너리를 업데이트합니다.

| Direction | Data |
| --- | --- |
| Input | `--force`; `OPENSTOCK_INSTALL_DIR`, `OPENSTOCK_RELEASE_API_URL`, `OPENSTOCK_RELEASE_ASSET_SUFFIX` 선택 지정 |
| External IO | `GET https://git.hananakick.cc/api/v1/repos/Autotrade/openstock/releases/latest`; selected release asset download |
| File write | 현재 실행 바이너리 디렉터리 또는 `OPENSTOCK_INSTALL_DIR`의 `openstock` 바이너리 |
| Output fields | `release_api_url`, `current_version`, `latest_version`, `release_tag`, `release_url`, `asset_name`, `asset_url`, `install_dir`, `status`, `stdout`, `stderr` |
| Raw | `null` |
| Side effect | 최신 release 버전이 현재 버전보다 크거나 `--force`이면 설치된 바이너리를 교체합니다. `~/.config/openstock` 설정/캐시는 보존합니다. |

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
| Config write | `~/.config/openstock/.env`에 `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCESS_TOKEN`, `KIS_ACCESS_TOKEN_EXPIRED_AT`, `KIS_TOKEN_TYPE`, `KIS_ACCESS_TOKEN_EXPIRES_IN` |
| External IO | `POST https://openapi.koreainvestment.com:9443/oauth2/tokenP` only when token is missing/expired or `--force` is used |
| Output fields | `broker`, `status`, `force`, `credential_source`, `token_storage`, `side_effect` |
| Side effect | writes auth state to `~/.config/openstock/.env` |

### `openstock api call <endpoint> --param KEY=VALUE`

KIS endpoint를 직접 GET 호출합니다. `tr_id` 파라미터는 HTTP header로 전송하고 나머지는 query string으로 전송합니다.

| Direction | Data |
| --- | --- |
| Input | `endpoint`, repeated `--param KEY=VALUE` |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET` |
| External IO | KIS endpoint GET |
| File IO | `~/.config/openstock/.env` read |
| Output fields | `broker`, `endpoint`, `params`, `request_semantics`, `response`, `response_semantics` |
| Raw | parsed KIS response. `api call`은 직접 호출 명령이므로 원본 응답을 `raw`에도 유지합니다. |
| Side effect | endpoint에 따라 다름. typed command 사용 권장 |

### `openstock search <query>`

Naver 모바일 주식 검색 API로 종목 후보를 검색합니다.

| Direction | Data |
| --- | --- |
| Input | `query`: 종목명 또는 종목코드 |
| External IO | `GET https://m.stock.naver.com/front-api/search/autoComplete` |
| File IO | 없음 |
| Output fields | `provider`, `query`, `stocks` |
| Raw | `null` |
| Side effect | 없음 |

`stocks` 항목은 가능한 경우 `code`, `name`, `market`, `market_code`, `nation_code`, `category`, `reuters_code`, `url`을 포함합니다.

### `openstock score set <symbol> <score>`

종목 ID에 0~100점 평가 점수를 저장하거나 갱신합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`, `score` |
| File read/write | `~/.config/openstock/scores.json` |
| Output fields | `path`, `symbol`, `score`, `updated_at_unix` |
| Raw | `null` |
| Side effect | 종목 평가 점수 파일을 생성하거나 갱신합니다. |

### `openstock score get <symbol>`

종목 ID의 저장된 평가 점수를 조회합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol` |
| File read | `~/.config/openstock/scores.json` |
| Output fields | `path`, `symbol`, `score`, `updated_at_unix` |
| Raw | `null` |
| Side effect | 없음 |

### `openstock score list`

저장된 종목 평가 점수 목록을 점수 내림차순으로 조회합니다.

| Direction | Data |
| --- | --- |
| Input | 없음 |
| File read | `~/.config/openstock/scores.json` |
| Output fields | `path`, `count`, `scores` |
| Raw | `null` |
| Side effect | 없음 |

### `openstock score delete <symbol>`

종목 ID의 저장된 평가 점수를 삭제합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol` |
| File read/write | `~/.config/openstock/scores.json` |
| Output fields | `path`, `symbol`, `deleted`, `removed` |
| Raw | `null` |
| Side effect | 저장된 점수가 있으면 해당 기록을 삭제합니다. |

### `openstock universe sync [--force]`

KIND 상장법인목록을 받아 stock universe cache를 갱신합니다.

| Direction | Data |
| --- | --- |
| Input | `--force` |
| External IO | `GET https://kind.krx.co.kr/corpgeneral/corpList.do?method=download&searchType=13` |
| File write | `~/.config/openstock/cache/universe/kind/latest.json`, `meta.json`, `YYYY-MM-DD.json` |
| Output fields | `source`, `cache_date`, `refreshed_at`, `refreshed`, `stock_count`, `counts_by_market` |
| Raw | `null` |
| Side effect | writes/updates local cache; prunes old snapshots |

### `openstock universe status`

| Direction | Data |
| --- | --- |
| Input | 없음 |
| File read | `~/.config/openstock/cache/universe/kind/latest.json`, `meta.json` |
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
| Raw | `null` |
| Side effect | 없음 |

### `openstock market history <symbol> --from YYYYMMDD --to YYYYMMDD [--period D|W|M|Y] [--raw-price]`

KIS 기간별 OHLCV를 조회합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`, `--from`, `--to`, `--period`, `--raw-price` |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET` |
| External IO | `GET /uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice`, TR ID `FHKST03010100` |
| Output fields | `broker`, `symbol`, `period`, `date_range`, `adjusted`, `count`, `candles`, `summary` |
| Raw | `null` |
| Side effect | 없음 |

`candles`는 날짜 오름차순이며 각 항목은 `date`, `open`, `high`, `low`, `close`, `volume`, `trading_value`, `change`, `change_sign`, `change_rate`를 포함합니다. 숫자는 원본 정밀도 보존을 위해 문자열로 반환합니다.

### `openstock dart sync [--force]`

OpenDART 고유번호 ZIP/XML을 받아 상장사 `stock_code` to `corp_code` cache를 갱신합니다.

| Direction | Data |
| --- | --- |
| Input | `--force` |
| Env read | `OPENDART_API_KEY` |
| External IO | `GET https://opendart.fss.or.kr/api/corpCode.xml` |
| File write | `~/.config/openstock/cache/opendart/corp_codes.json`, `corp_codes_meta.json` |
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
| Raw | `null` |
| Side effect | cache refresh 가능 |

`disclosures` 항목은 OpenDART가 반환하는 `corp_cls`, `corp_code`, `corp_name`, `stock_code`, `rcept_no`, `report_nm`, `rcept_dt`, `flr_nm`, `rm` 등을 포함합니다.

### `openstock dart document <rcept_no> [--force] [--max-chars N]`

접수번호로 공시서류 원본 ZIP을 다운로드하고 XML 본문 텍스트를 추출합니다.

| Direction | Data |
| --- | --- |
| Input | `rcept_no`: 14자리 접수번호, `--force`, `--max-chars` |
| Env read | `OPENDART_API_KEY` |
| External IO | `GET https://opendart.fss.or.kr/api/document.xml` |
| File read/write | `~/.config/openstock/cache/opendart/documents/{rcept_no}.zip` |
| Output fields | `rcept_no`, `source`, `viewer_url`, `cached`, `zip_path`, `zip_bytes`, `files`, `text`, `text_chars`, `truncated` |
| Raw | `null` |
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
| Raw | `null` |
| Side effect | cache refresh/write 가능 |

### `openstock account status`

KIS 계좌 잔고와 보유종목을 조회합니다.

| Direction | Data |
| --- | --- |
| Input | 없음 |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS `GET /uapi/domestic-stock/v1/trading/inquire-balance` |
| Output fields | `broker`, `account`, `balance`, `holdings` |
| Raw | `null` |
| Side effect | 없음 |

### `openstock order buy <symbol> --qty QTY (--price PRICE|--market)`

국내주식 매수 주문을 전송합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`, `--qty`, one of `--price` or `--market` |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS `POST /uapi/domestic-stock/v1/trading/order-cash`, TR ID `TTTC0802U` |
| Output fields | `broker`, `side`, `symbol`, `qty`, `order_type`, `price`, `order` |
| Raw | `null` |
| Side effect | financial_order: 실전 매수 주문 |

### `openstock order sell <symbol> --qty QTY (--price PRICE|--market)`

국내주식 매도 주문을 전송합니다.

| Direction | Data |
| --- | --- |
| Input | `symbol`, `--qty`, one of `--price` or `--market` |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS `POST /uapi/domestic-stock/v1/trading/order-cash`, TR ID `TTTC0801U` |
| Output fields | `broker`, `side`, `symbol`, `qty`, `order_type`, `price`, `order` |
| Raw | `null` |
| Side effect | financial_order: 실전 매도 주문 |

### `openstock order status [order_no] [--from YYYYMMDD] [--to YYYYMMDD]`

주문 및 체결 내역을 조회합니다.

| Direction | Data |
| --- | --- |
| Input | optional `order_no`, optional date range |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS `GET /uapi/domestic-stock/v1/trading/inquire-daily-ccld` |
| Output fields | `broker`, `account`, `order_no`, `orders`, `summary` |
| Raw | `null` |
| Side effect | 없음 |

### `openstock cache status`

| Direction | Data |
| --- | --- |
| Input | 없음 |
| File read | `~/.config/openstock/cache` recursively |
| Output fields | `root`, `exists`, `total_files`, `total_bytes`, `namespaces` |
| Side effect | 없음 |

### `openstock cache prune [--dry-run]`

캐시 보존 정책을 적용합니다.

| Direction | Data |
| --- | --- |
| Input | `--dry-run` |
| File read | `~/.config/openstock/cache` directories |
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
