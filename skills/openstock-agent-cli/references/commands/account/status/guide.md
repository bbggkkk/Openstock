# `openstock account status`

Purpose: query balance and holdings for the configured KIS account.

Usage:

```bash
openstock account status
```

Reads: `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCOUNT`.

External IO: KIS balance inquiry.

Output fields: `broker`, `account`, `balance`, `holdings`.

Agent rule: run before any trade recommendation or live order.
