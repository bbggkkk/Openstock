# `openstock order`

## Purpose

Place and query live KIS domestic stock orders.

## Subcommands

| Command | Purpose |
| --- | --- |
| `openstock order buy <symbol> --qty QTY (--price PRICE|--market)` | Place a live buy order. |
| `openstock order sell <symbol> --qty QTY (--price PRICE|--market)` | Place a live sell order. |
| `openstock order status [order_no] [--from YYYYMMDD] [--to YYYYMMDD]` | Query order/execution status. |

## Safety

`buy` and `sell` are live financial orders. Do not run them unless the user explicitly asks for a real order in the current task.

Before placing an order:

1. Run `openstock account status`.
2. Confirm the symbol with `search` or `market`.
3. Confirm side, quantity, order type, and price.
4. Prefer `order status` for verification after order placement.

## IO

| Direction | Data |
| --- | --- |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS order and order inquiry endpoints. |
| Side effect | `buy` and `sell` place live orders. `status` is read-only. |

## Output Fields

| Field | Meaning |
| --- | --- |
| `broker` | Broker used for order. |
| `side` | `buy` or `sell`. |
| `symbol` | Ordered symbol. |
| `qty` | Order quantity. |
| `order_type` | `limit` or `market`. |
| `price` | Limit price or market-order price representation. |
| `order` | Broker order result. |
| `account` | Account queried for status. |
| `order_no` | Requested order number filter. |
| `orders` | Order/execution list. |
| `summary` | Broker summary values. |

## Agent Notes

For analysis-only tasks, never use `buy` or `sell`. Use `order status` only when order history is relevant.
