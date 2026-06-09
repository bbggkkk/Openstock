# `openstock account`

## Purpose

Read KIS account state for the configured live account.

## Subcommands

| Command | Purpose |
| --- | --- |
| `openstock account status` | Show balance and holdings. |

## `account status`

### Usage

```bash
openstock account status
```

### IO

| Direction | Data |
| --- | --- |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT` |
| External IO | KIS balance inquiry endpoint. |
| File IO | `~/.config/openstock/.env` read. |
| Side effect | none |

### Output Fields

| Field | Meaning |
| --- | --- |
| `broker` | Broker API used for inquiry. |
| `account` | Queried account number. |
| `balance` | Cash, total evaluated asset amount, profit/loss, and account summary values. |
| `holdings` | Currently held stock list. |

## Agent Notes

Use this before any proposed trade to check cash, holdings, and orderable quantity context.
