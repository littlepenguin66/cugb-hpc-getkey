# troubleshooting

## quick triage

when `ghpc` fails, first classify the failure:

1. did it fail before login submission?
2. did login succeed but token retrieval fail?
3. did token retrieval succeed but key download fail?
4. did local file writing or permissions fail?

run with `--verbose` first:

```bash
ghpc --verbose
```

that tells you which numbered step in `src/login.rs` failed.

## common problems

### failed to read password

symptom:

- error mentions `Failed to read password`

what it means:

- the cli needed an interactive password prompt
- but the current terminal session could not provide a usable tty for hidden input

likely causes:

- running inside a non-interactive shell wrapper
- invoking the command from a tool that does not expose a real tty
- terminal permissions blocked secure password input

what to check:

- whether `HPC_PASSWORD` is already set
- whether you passed `--password`
- whether `--password-stdin` would be a better fit
- whether the command is running in a real terminal

next actions:

- run the command directly in a terminal
- provide the password through `HPC_PASSWORD` for non-interactive use
- provide the password through `--password-stdin` for script-friendly use
- avoid wrappers that cannot open a tty for password input

### password read from stdin was empty

symptom:

- error mentions `Password read from stdin was empty`

what it means:

- `--password-stdin` was used
- but stdin did not contain any password bytes after trimming the trailing line ending

what to check:

- whether the pipeline actually writes a password
- whether the upstream command produced an empty line

next actions:

- print the password value into the pipe explicitly
- confirm the shell pipeline is not swallowing the input
- retry with a known non-empty test value first

### failed to get execution token

symptom:

- error mentions `Failed to get execution token`

what it means:

- the cas login page was fetched
- but the html no longer matched the expected hidden input pattern

likely causes:

- the upstream login page structure changed
- the page returned an unexpected error page instead of the real form
- a network gateway, vpn, or captive portal altered the response

what to check:

```bash
curl -I "https://hpc.cugb.edu.cn/sso/login"
```

next actions:

- inspect whether the upstream login page changed
- update the regex in `src/login.rs`
- retry with a stable network path

### encryption failure

symptom:

- error happens around password encryption
- stderr may mention fallback public key usage

what it means:

- the tool could not safely use the upstream rsa public key
- or the parsed key could not be used for encryption

likely causes:

- `login.js` changed format
- upstream served a malformed key
- the default fallback key is outdated

what to check:

- whether the `login.js` file still contains `var key = '...'`
- whether the fallback key still matches the server expectation

next actions:

- inspect `src/crypto.rs`
- update the extraction regex if upstream formatting changed
- update `DEFAULT_PUBLIC_KEY` if the server rotated permanently

### login failed, status: ...

symptom:

- error mentions `Login failed, status: ...`

what it means:

- the login post did not return one of the success-shaped responses the tool expects

likely causes:

- invalid username or password
- server-side auth flow changed
- anti-bot or upstream validation changed request requirements

what to check:

- confirm the credentials work in the browser
- retry with `--verbose`
- verify request headers in `src/login.rs` still resemble a browser enough for the server

### failed to get token

symptom:

- login redirects appear to work
- but token fetch fails afterward

what it means:

- cas session was not translated into a usable application session
- or the token response schema changed

likely causes:

- missing or incomplete cookies after redirect handling
- token endpoint returned a different json shape
- upstream application changed auth expectations

what to check:

- verify the redirect chain still produces the expected cookies
- inspect `getCurrentUserInfo.action` response
- compare with browser network traces if needed

### failed to get private key

symptom:

- error text starts with `Failed to get private key`

what it means:

- the gridview endpoint rejected the token
- or it returned a failure-shaped json payload

likely causes:

- cached token is expired or no longer accepted
- fresh login succeeded but the gridview backend still rejected the token
- endpoint response shape changed

what to check:

- try `ghpc --force --verbose`
- verify whether the failure happens only on cached tokens or also after fresh login
- inspect the returned `msg` field if present

important behavior:

- current code will automatically retry with a fresh login if cached token download fails
- if a fresh login also fails, the program exits non-zero

### cache status is expired

symptom:

- `ghpc --status` reports `expired`

what it means:

- local token exists, but current time is past `expires_at`

what to do:

- run `ghpc` normally and let it re-login
- or force refresh explicitly:

```bash
ghpc --force
```

### cache status is invalid

symptom:

- `ghpc --status` reports `invalid`

what it means:

- the cache file exists
- but its json no longer matches the expected format

likely causes:

- manual edits to `~/.hpc-login-cache.json`
- a partially written file after an interrupted run
- old or unrelated data placed at the cache path

what to check:

- the contents of `~/.hpc-login-cache.json`
- whether the file is valid json

next actions:

- remove the cache file
- run `ghpc --force`
- let the cli rewrite a fresh cache file

### cache exists but is ignored

symptom:

- cache file exists
- but the tool still performs a fresh login

likely causes:

- cached username does not match the requested username
- token is expired
- cache json is malformed
- cached token download failed and the tool fell back to fresh login

what to check:

- `ghpc --status`
- contents of `~/.hpc-login-cache.json`
- whether you changed usernames between runs

### private key file is not updated

symptom:

- command exits with error
- `~/.hpckey` is missing or still old

what it means:

- network flow may have succeeded partially
- but the final download or local write failed

likely causes:

- download endpoint failure
- home directory resolution failure
- filesystem permission issue

what to check:

- whether `HOME` is set correctly
- write permissions for your home directory
- existing permissions and ownership of `~/.hpckey`

### permission problems on macos or linux

symptom:

- key file or cache file exists but permissions look too open
- or chmod/write fails

what the tool does:

- forces both cache and key files to `0600` on unix

what to check manually:

```bash
ls -l ~/.hpckey ~/.hpc-login-cache.json
```

expected mode:

- owner read/write only

## safe recovery steps

if you want a clean local reset:

1. inspect current cache status with `ghpc --status`
2. back up `~/.hpckey` if needed
3. remove the cache file
4. run `ghpc --force --verbose`

example:

```bash
rm -f ~/.hpc-login-cache.json
ghpc --force --verbose
```

## when to update the code

you probably need a code update if:

- the login page html changed
- the javascript public key format changed
- the redirect chain changed shape
- the token json shape changed
- the key download endpoint changed response format

the most likely files to touch are:

- `src/login.rs`
- `src/crypto.rs`
- `src/cache.rs`

## useful local commands

```bash
ghpc --status
ghpc --force --verbose
cargo build --release
```
