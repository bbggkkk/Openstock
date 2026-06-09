# `openstock dart corp`

Purpose: resolve a KRX stock symbol to an OpenDART corp code.

Usage:

```bash
openstock dart corp 005930
```

Input: `symbol`.

Reads: OpenDART corp code cache; refreshes on miss when needed.

Output fields: `stock_code`, `corp_code`, `corp_name`, `modify_date`.

Agent rule: use when an OpenDART command needs an explicit corp code.
