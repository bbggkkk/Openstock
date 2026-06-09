# `openstock search`

## Purpose

Search stock candidates by name or symbol using Naver mobile stock search.

## Usage

```bash
openstock search 삼성전자
openstock search 005930
```

## Input

| Input | Meaning |
| --- | --- |
| `query` | Stock name or stock code. |

## IO

| Direction | Data |
| --- | --- |
| External IO | `GET https://m.stock.naver.com/front-api/search/autoComplete` |
| File IO | none |
| Side effect | none |

## Output Fields

| Field | Meaning |
| --- | --- |
| `provider` | External data provider. |
| `query` | Query submitted by the user/agent. |
| `stocks` | Matching stock candidates. Items can include code, name, market, market code, nation code, category, Reuters code, and URL. |

## Agent Notes

Use this to resolve a human company name to a tradable symbol before calling `market`, `market history`, or `dart`.
