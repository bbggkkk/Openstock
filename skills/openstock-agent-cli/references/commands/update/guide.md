# `openstock update`

## Purpose

Update the installed `openstock` binary from the latest Gitea release asset.

## Usage

```bash
openstock update
openstock update --force
```

## Input

| Input | Meaning |
| --- | --- |
| `--force` | Reinstall from the latest release asset even when the current version matches. |
| `OPENSTOCK_INSTALL_DIR` | Optional install directory override. |
| `OPENSTOCK_RELEASE_API_URL` | Optional release API URL override. |
| `OPENSTOCK_RELEASE_ASSET_SUFFIX` | Optional asset suffix override. |

## IO

| Direction | Data |
| --- | --- |
| External IO | Gitea latest release API and selected release asset download. |
| File write | Installed `openstock` binary. |
| Side effect | Replaces the installed binary when newer or forced. |

## Output Fields

| Field | Meaning |
| --- | --- |
| `release_api_url` | Gitea release API endpoint used for update discovery. |
| `current_version` | Version of the running binary. |
| `latest_version` | Version parsed from latest release tag. |
| `release_tag` | Latest release tag. |
| `release_url` | Human-readable release page. |
| `asset_name` | Release asset selected for installation. |
| `asset_url` | Asset download URL. |
| `install_dir` | Directory where the binary is installed. |
| `status` | `updated` or `up_to_date`. |
| `stdout` | Installer stdout. |
| `stderr` | Installer stderr. |

## Agent Notes

Use `--force` only when the user asks to reinstall or when verifying release assets.
