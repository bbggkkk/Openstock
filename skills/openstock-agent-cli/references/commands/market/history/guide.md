# `openstock market history`

Purpose: query OHLCV price history for a stock.

Usage:

```bash
openstock market history 005930 --from 20260101 --to 20260609
openstock market history 005930 --from 20260101 --to 20260609 --period W
```

Inputs: `symbol`, `--from`, `--to`, optional `--period`, `--raw-price`.

Reads: KIS credentials and access token.

External IO: KIS price history endpoint.

Output fields: `broker`, `symbol`, `period`, `date_range`, `adjusted`, `count`, `candles`, `summary`.

Agent rule: use for price movement evidence; keep numeric strings intact unless calculating.
