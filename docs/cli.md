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
3. interactive prompt if either value is missing

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
Cache status: valid
Username: your_user
Expires: 2026-04-11 12:34:56
```

or:

```text
No cache
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

enables debug output and prints the token to stdout after success.

current effect:

- prints step-by-step debug messages to stderr
- prints `✓ Using cached token` when success came from cache
- prints the token to stdout on success

example:

```bash
ghpc --verbose
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
- token printed to stdout

### quiet successful run

with `--quiet`, expect:

- fewer informational messages
- no automatic token print unless you also changed the code path to do so

### failure output

on failure the program prints:

```text
Operation failed: ...
```

and exits non-zero.

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

### force refresh

```bash
ghpc --force
```

### inspect cache only

```bash
ghpc --status
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

### current metadata mismatch

the clap help metadata in `src/cli.rs` is not fully aligned with the crate version and binary naming in `Cargo.toml`.

practically:

- users still run the binary as `ghpc`
- but generated help/version text may lag behind package metadata until that file is updated
