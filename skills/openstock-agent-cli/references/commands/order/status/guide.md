# `openstock order status`

Purpose: query KIS order and execution status.

Usage:

```bash
openstock order status
openstock order status 1234567890 --from 20260601 --to 20260609
```

Inputs: optional `order_no`, `--from`, `--to`.

External IO: KIS order inquiry endpoint.

Output fields: `broker`, `account`, `order_no`, `orders`, `summary`.

Agent rule: read-only; use after order placement or when auditing order history.
