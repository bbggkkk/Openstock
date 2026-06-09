# `openstock version`

## Purpose

Return the installed `openstock` package name and version.

## Usage

```bash
openstock version
```

## Input

No arguments.

## IO

| Direction | Data |
| --- | --- |
| External IO | none |
| File IO | none |
| Side effect | none |

## Output Fields

| Field | Meaning |
| --- | --- |
| `name` | Cargo package/program name. |
| `version` | Version of the running `openstock` binary. |

## Agent Notes

Use this before update checks or when reporting runtime provenance.
