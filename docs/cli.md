# cli

## command

the built binary is:

```bash
ghpc
```

you can also run it from the cargo build output:

```bash
./target/release/ghpc
```

## inputs

the cli needs:

- hpc username
- hpc password

it resolves them in this order:

1. `--username` / `--password`
2. `HPC_USERNAME` / `HPC_PASSWORD`
3. interactive prompt for whichever value is still missing

if `--password-stdin` is present:

- the password is read from stdin instead of `--password` or `HPC_PASSWORD`
- username must already be available through `--username` or `HPC_USERNAME`

## flags

### `-u, --username <username>`

sets the hpc username directly.

example:

```bash
ghpc -u your_user
```

### `-p, --password <password>`

sets the hpc password directly.

example:

```bash
ghpc -u your_user -p your_password
```

warning:

- convenient, but not recommended for long-term use
- command line passwords can leak into shell history and process listings

### `--password-stdin`

reads the hpc password from stdin.

example:

```bash
printf '%s\n' "$HPC_PASSWORD" | ghpc --username your_user --password-stdin
```

when to use it:

- non-interactive scripts
- shells where you do not want the password in process args
- environments where hidden tty input is unavailable

### `-f, --force`

skips cache reuse and always performs a fresh login.

example:

```bash
ghpc --force
```

when to use it:

- cached token seems stale
- login state drifted
- you want to force regeneration of local auth state

### `-s, --status`

prints cache metadata only.

example:

```bash
ghpc --status
```

typical output:

```text
Cache: valid
Cache path: /home/you/.hpc-login-cache.json
Cache file: present
Cache mode: 600
Cache modified: 2026-04-11 11:11:11
Cache size: 475 bytes
Username: your_user
Expires: 2026-04-11 12:34:56
Remaining: 1h 23m 45s
Key path: /home/you/.hpckey
Key file: present
Key mode: 600
Key modified: 2026-04-11 11:11:11
Key size: 2610 bytes
```

or:

```text
Cache: invalid
Cache path: /home/you/.hpc-login-cache.json
Cache file: present
Cache mode: 600
Cache modified: 2026-04-11 11:11:11
Cache size: 91 bytes
Cache error: Failed to parse cache file
Key path: /home/you/.hpckey
Key file: present
Key mode: 600
Key modified: 2026-04-11 11:11:11
Key size: 2610 bytes
```

or:

```text
Cache: missing
Cache path: /home/you/.hpc-login-cache.json
Cache file: missing
Key path: /home/you/.hpckey
Key file: missing
```

### `-q, --quiet`

suppresses informational output from the downloader.

current effect:

- hides messages such as `Private key saved to: ...`
- does not make the program print the token

example:

```bash
ghpc --quiet
```

### `-v, --verbose`

enables debug output.

current effect:

- prints step-by-step debug messages to stderr
- prints `✓ Using cached token` when success came from cache
- prints raw detail lines for normalized failures

example:

```bash
ghpc --verbose
```

### `--print-token`

prints the resolved token on success.

current effect:

- token output is opt-in instead of being bundled into `--verbose`
- works in normal text mode
- in `--json` mode the token is included in the success object only when this flag is present

example:

```bash
ghpc --force --print-token
```

### `--json`

emits structured json output.

current effect:

- `ghpc --status --json` prints cache and key metadata as json
- normal successful runs print a small json success object
- `--print-token --json` adds the resolved token field to the success object
- failures print a json error object and exit non-zero
- human-oriented info and debug logs are suppressed while json mode is active

example:

```bash
ghpc --status --json
```

## environment variables

### `HPC_USERNAME`

sets the default username.

### `HPC_PASSWORD`

sets the default password.

example:

```bash
export HPC_USERNAME=your_user
export HPC_PASSWORD=your_password
ghpc
```

## output behavior

### default successful run

default success output is minimal.

normally you will see:

- `Private key saved to: /home/.../.hpckey`

you will not normally see:

- the token
- cache-hit messages
- step-by-step debug logs

### verbose successful run

with `--verbose`, expect:

- debug step logs to stderr
- cache-hit marker if applicable
- raw detail lines for normalized failures

### quiet successful run

with `--quiet`, expect:

- fewer informational messages
- `--quiet` cannot be combined with `--verbose`
- `--print-token` still prints the token because it is an explicit data output

### failure output

on failure the program prints:

```text
error: ...
hint: use --verbose for step logs
```

and exits non-zero.

### json mode

with `--json`, the cli switches to machine-readable output.

that means:

- `--status` writes a json object to stdout
- successful non-status runs write a json object to stdout
- failures write a json object to stderr
- normalized errors keep a short `error` field and store raw backend detail separately
- `--verbose` step logs are not mixed into json output

## common command patterns

### normal interactive use

```bash
ghpc
```

### env-var driven use

```bash
export HPC_USERNAME=your_user
export HPC_PASSWORD=your_password
ghpc
```

### stdin password flow

```bash
printf '%s\n' "$HPC_PASSWORD" | ghpc --username your_user --password-stdin
```

### force refresh

```bash
ghpc --force
```

### inspect cache only

```bash
ghpc --status
```

### inspect cache as json

```bash
ghpc --status --json
```

### print token explicitly

```bash
ghpc --force --print-token
```

### debug a failing run

```bash
ghpc --force --verbose
```

## local files touched by the cli

### cache file

path:

- `~/.hpc-login-cache.json`

purpose:

- stores short-lived token cache

### private key file

path:

- `~/.hpckey`

purpose:

- stores the downloaded ssh private key

## behavior notes

### cache semantics

- cache is only used when username matches
- cache is ignored when expired
- cache is ignored when `--force` is set
- if cached token download fails, the cli falls back to a fresh login automatically

### key download semantics

- only a success-shaped api response writes `~/.hpckey`
- failure-shaped responses now abort the command instead of pretending success

### cli metadata

the clap help metadata in `src/cli.rs` now follows the crate metadata from `Cargo.toml`.

practically:

- users run the binary as `ghpc`
- generated help/version text stays aligned with package metadata automatically
- `--help` groups flags into `input`, `output`, and `mode`
- `--version` prints the current version together with a short description
