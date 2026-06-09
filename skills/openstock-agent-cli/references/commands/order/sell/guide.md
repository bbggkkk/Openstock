# `openstock order sell`

Purpose: place a live KIS domestic stock sell order.

Usage:

```bash
openstock order sell 005930 --qty 1 --price 70000
openstock order sell 005930 --qty 1 --market
```

Inputs: `symbol`, `--qty`, one of `--price` or `--market`.

External IO: KIS live order endpoint.

Side effect: live financial sell order.

Output fields: `broker`, `side`, `symbol`, `qty`, `order_type`, `price`, `order`.

Agent rule: only run after explicit current-user approval for a real order.
