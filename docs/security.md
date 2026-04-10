# security

## overview

`ghpc` handles three sensitive data classes:

1. hpc username
2. hpc password
3. downloaded ssh private key

it also stores a short-lived token cache locally.

the main security question is not only "what is secret", but also "where does it live" and "for how long".

## data boundary summary

### `HPC_USERNAME`

classification:

- low sensitivity compared with password and key
- still personal account data

where it may appear:

- shell environment
- process environment of the running command
- terminal history only if exported via commands you save in shell startup files or scripts

### `HPC_PASSWORD`

classification:

- high sensitivity

where it may appear:

- process arguments if passed with `-p`
- shell history if typed directly on the command line
- shell environment if exported as `HPC_PASSWORD`
- process memory during login

the tool itself does not write the password to disk, but how you invoke it still matters.

### `~/.hpc-login-cache.json`

classification:

- sensitive
- contains a reusable token, not the password

what it stores:

- username
- token
- `expires_at`
- `created_at`

security model:

- short-lived local convenience cache
- unix permissions are tightened to `0600`
- if the file is stolen during its validity window, an attacker may be able to reuse the token

### `~/.hpckey`

classification:

- highest sensitivity in this project

what it contains:

- the downloaded ssh private key for hpc access

security model:

- the tool writes it locally
- unix permissions are tightened to `0600`
- compromise of this file is equivalent to compromise of the corresponding ssh identity while it remains valid

## why `-p` is a bad default

passing the password with:

```bash
ghpc -u your_user -p your_password
```

is convenient, but it has avoidable exposure risks.

### risk 1: shell history

many shells record full command lines in history files.

that means your password may end up in:

- `~/.zsh_history`
- shell sync or backup tools
- copied terminal logs

### risk 2: process list exposure

command line arguments can be visible to:

- `ps`
- monitoring tools
- debugging tools
- other local processes depending on system permissions

### risk 3: accidental reuse

people often paste the same command again later, share snippets, or leave it in scripts.

that turns a one-time convenience choice into long-lived credential exposure.

## preferred ways to provide credentials

### best interactive option

run `ghpc` with no password flag and enter the password at the prompt:

```bash
ghpc
```

advantages:

- password is not placed in shell history
- password is not present in process arguments

### acceptable automation option

use environment variables when you need non-interactive execution:

```bash
export HPC_USERNAME=your_user
export HPC_PASSWORD=your_password
ghpc
```

this is still sensitive, but usually safer than `-p` because the secret does not appear in the argument list.

for better hygiene:

- avoid persisting `HPC_PASSWORD` in shared shell config files
- prefer one-shot shells, local secret managers, or ephemeral export in the current session only

## local file protections

the code attempts to set both:

- `~/.hpc-login-cache.json`
- `~/.hpckey`

to mode `0600` on unix systems.

you should still verify that occasionally:

```bash
ls -l ~/.hpckey ~/.hpc-login-cache.json
```

expected result:

- readable and writable only by the current user

## ai usage boundary

this repository ships an ai skill package under `.skills/cugb-hpc-getkey/`.

security implication:

- the skill helps an assistant understand the workflow
- it does not make sharing credentials safe

practical rule:

- do not send your real password or private key to an ai system unless you fully trust the runtime, logging path, storage policy, and operators behind it

## cache safety guidance

the token cache is a convenience feature, not a trust boundary.

good practices:

- use `--force` if you suspect token misuse or state drift
- remove `~/.hpc-login-cache.json` if you want to invalidate local cached state
- do not copy the cache file between machines

## key safety guidance

good practices for `~/.hpckey`:

- never commit it
- never send it over chat
- do not move it into synced folders casually
- rotate or re-download it if you suspect exposure

## operational recommendations

for normal daily use:

- use interactive prompt or env vars
- avoid `-p`
- keep `~/.hpckey` and cache file owner-only

for debugging:

- prefer `--verbose`
- avoid printing secrets into shared terminals or logs

for automation:

- use narrowly scoped local env vars
- do not store plaintext passwords in reusable scripts unless you accept that risk
