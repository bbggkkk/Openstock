# `openstock dart`

## Purpose

Query OpenDART listed-company mappings, disclosure lists, and disclosure documents.

## Subcommands

| Command | Purpose |
| --- | --- |
| `openstock dart sync [--force]` | Refresh OpenDART corp code cache. |
| `openstock dart status` | Show corp code cache status. |
| `openstock dart corp <symbol>` | Resolve KRX symbol to DART corp code. |
| `openstock dart disclosures [symbol] [--corp-code CODE] [--from YYYYMMDD] [--to YYYYMMDD] [--corp-cls Y|K|N|E] [--page-no N] [--page-count N]` | Query disclosure list. |
| `openstock dart document <rcept_no> [--force] [--max-chars N]` | Fetch disclosure document text. |
| `openstock dart show <symbol> [--from YYYYMMDD] [--to YYYYMMDD] [--index N] [--page-count N] [--force] [--max-chars N]` | Query disclosures for a symbol and fetch one selected document. |

## IO

| Direction | Data |
| --- | --- |
| Env read | `OPENDART_API_KEY` for OpenDART API calls. |
| External IO | OpenDART corp code, list, and document endpoints. |
| File read/write | `~/.config/openstock/cache/opendart`. |
| Side effect | `sync`, `document`, and `show` can write cache. |

## Important Fields

| Field | Meaning |
| --- | --- |
| `stock_code` | KRX stock code. |
| `corp_code` | OpenDART 8-digit company code. |
| `corp_name` | OpenDART company name. |
| `modify_date` | Corp code mapping modification date. |
| `query` | Disclosure query sent to OpenDART. |
| `resolved` | Symbol/corp code resolution details. |
| `disclosures` | Disclosure list. |
| `selected` | Disclosure selected by `--index`. |
| `document` | Disclosure document metadata and extracted text. |

## Agent Notes

Use `dart show` for the common workflow: symbol to disclosures to selected document. Use `dart disclosures` when only the list is needed.
