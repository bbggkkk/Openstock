# `openstock api`

## Purpose

Manage broker API metadata, KIS login, and raw endpoint calls.

## Subcommands

| Command | Purpose |
| --- | --- |
| `openstock api list` | Show registered broker APIs and their capabilities. |
| `openstock api login [--appkey KEY] [--appsecret SECRET] [--force]` | Issue or reuse a KIS access token. |
| `openstock api call <endpoint> --param KEY=VALUE` | Direct KIS GET endpoint call with token headers. |

## `api list`

### IO

| Direction | Data |
| --- | --- |
| External IO | none |
| File IO | none |
| Side effect | none |

### Output Fields

| Field | Meaning |
| --- | --- |
| `count` | Number of registered broker APIs. |
| `apis` | Broker metadata, credential requirements, capability catalog, IO contract, and side effects. |

## `api login`

### Usage

```bash
openstock api login
openstock api login --force
```

### IO

| Direction | Data |
| --- | --- |
| Env read | `KIS_APPKEY`, `KIS_APPSECRET`, `KIS_ACCESS_TOKEN`, `KIS_ACCESS_TOKEN_EXPIRED_AT` |
| Config write | `~/.config/openstock/.env` auth state. |
| External IO | KIS token endpoint only when token is missing, expired, or `--force` is used. |
| Side effect | Writes auth token state. |

### Output Fields

| Field | Meaning |
| --- | --- |
| `broker` | Broker API used for login. |
| `status` | Login result. |
| `force` | Whether token refresh was forced. |
| `credential_source` | Whether credentials came from CLI arguments or config. |
| `token_storage` | Token storage path and env keys. |
| `side_effect` | Auth-state write indicator. |

## `api call`

### Usage

```bash
openstock api call /uapi/... --param tr_id=... --param KEY=VALUE
```

### IO

| Direction | Data |
| --- | --- |
| Env read | `KIS_ACCESS_TOKEN`, `KIS_APPKEY`, `KIS_APPSECRET` |
| External IO | KIS endpoint GET. |
| File IO | `~/.config/openstock/.env` read. |
| Side effect | Endpoint dependent. |

### Output Fields

| Field | Meaning |
| --- | --- |
| `broker` | Broker used for the direct call. |
| `endpoint` | Requested endpoint. |
| `params` | Request parameters. |
| `request_semantics` | How params map to headers/query. |
| `response` | Parsed API response. |
| `response_semantics` | Explanation of response meaning. |

## Agent Notes

Prefer typed commands. Use `api call` only for inspection or endpoints not yet bound as typed commands.
