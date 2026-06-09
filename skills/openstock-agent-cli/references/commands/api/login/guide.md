# `openstock api login`

Purpose: issue or reuse a KIS live API access token.

Usage:

```bash
openstock api login
openstock api login --force
openstock api login --appkey KEY --appsecret SECRET
```

Inputs: optional `--appkey`, `--appsecret`, `--force`.

Reads: `KIS_APPKEY`, `KIS_APPSECRET`, existing token values.

Writes: `~/.config/openstock/.env` token state.

Output fields: `broker`, `status`, `force`, `credential_source`, `token_storage`, `side_effect`.

Agent rule: do not force refresh unless the user asks or an API call fails because the token is invalid.
