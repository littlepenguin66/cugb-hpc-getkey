# ghpc

[中文](README_ZH.md)

`ghpc` is a small rust cli for cugb hpc login automation. it logs in through cas sso, reuses a short-lived local token cache when possible, and downloads your ssh private key to `~/.hpckey`.

if you use the cugb hpc platform regularly, this tool exists for one reason: to turn an annoying browser workflow into one command.

## Why I Made This

the school hpc platform is usable, but not pleasant.

- the web ui is slow
- the browser eshell is not something i want to rely on
- the ssh key expires quickly
- the refresh flow is repetitive and easy to get tired of

every time the key expires, the usual path is:

1. open the portal
2. wait for the page to catch up
3. jump through the hpc pages
4. find the download action again
5. save the key locally

that is not hard, just annoying enough to deserve automation.

so this project keeps the workflow simple:

- log in
- reuse cache when valid
- refresh automatically when cache no longer works
- write the key to `~/.hpckey`

## Installation

```bash
git clone https://github.com/littlepenguin66/cugb-hpc-getkey.git
cd cugb-hpc-getkey
cargo build --release
```

the binary will be built at:

```bash
target/release/ghpc
```

if you want it available globally:

```bash
mv target/release/ghpc ~/.local/bin/
```

## Quick Start

interactive use:

```bash
ghpc
```

with environment variables:

```bash
export HPC_USERNAME=your_username
export HPC_PASSWORD=your_password
ghpc
```

with stdin password input:

```bash
printf '%s\n' "$HPC_PASSWORD" | ghpc --username your_username --password-stdin
```

force a fresh login:

```bash
ghpc --force
```

check cache status:

```bash
ghpc --status
```

get machine-readable status:

```bash
ghpc --status --json
```

debug a failing run:

```bash
ghpc --force --verbose
```

print the resolved token explicitly:

```bash
ghpc --force --print-token
```

## Options

| Flag | Description |
|------|-------------|
| `-u, --username` | HPC username, or use `HPC_USERNAME` |
| `-p, --password` | HPC password, or use `HPC_PASSWORD` |
| `--password-stdin` | Read the password from stdin |
| `-f, --force` | Skip cache and perform a fresh login |
| `-s, --status` | Print cache status only |
| `--json` | Emit JSON output |
| `--print-token` | Print the resolved token on success |
| `-q, --quiet` | Suppress informational output |
| `-v, --verbose` | Print debug logs |

## What It Does

1. requests the cugb hpc cas login page
2. extracts the `execution` token
3. encrypts your password with the upstream rsa public key
4. completes the sso redirect flow manually
5. requests a jwt token from the hpc api
6. downloads the ssh private key from gridview
7. writes the key to `~/.hpckey`
8. stores a short-lived cache in `~/.hpc-login-cache.json`

if cached token download fails, `ghpc` falls back to a fresh login automatically.

## Documentation

core docs:

- [architecture](docs/architecture.md)
- [cli](docs/cli.md)
- [security](docs/security.md)
- [troubleshooting](docs/troubleshooting.md)

release notes:

- [v2026.3.18](docs/release/v2026.3.18.md)
- [v2026.4.11](docs/release/v2026.4.11.md)
- [v26.4.12](docs/release/v26.4.12.md)

## Use with AI

this repo also ships an ai-oriented skill package in:

```text
.skills/cugb-hpc-getkey/
```

it helps coding assistants understand the login flow and local file layout.

security note:

- do not casually give your real hpc password or private key to an ai system
- prefer running the tool locally and only asking ai for help with setup or debugging

## Notes

- the upstream rsa public key may change over time
- if login suddenly breaks, check `docs/troubleshooting.md`
- if you care about safe credential handling, read `docs/security.md`

## License

see [LICENSE](LICENSE).
