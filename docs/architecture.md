# architecture

## overview

`ghpc` is a small rust cli that automates four connected concerns:

1. collect credentials
2. authenticate against cugb hpc cas sso
3. retrieve a jwt token for the gridview api
4. download and persist the ssh private key locally

the code is intentionally split by responsibility:

- `src/main.rs` orchestrates cli flow, cache usage, and fallback behavior
- `src/cli.rs` defines flags and env var bindings
- `src/login.rs` owns the network login flow and key download
- `src/crypto.rs` handles password encryption
- `src/cache.rs` reads and writes the local token cache
- `src/session.rs` serializes cookies for outgoing requests
- `src/types.rs` defines shared data structures

## end-to-end flow

### 1. input resolution

the program starts in `src/main.rs`.

- if `--status` is present, it skips login and prints cache metadata
- otherwise it resolves username and password from:
  - `--username` / `--password`
  - `HPC_USERNAME` / `HPC_PASSWORD`
  - interactive prompt fallback if either value is missing

the resolved values are wrapped into `LoginConfig`.

### 2. cache-first download flow

the main execution path is `execute_download_flow()` in `src/main.rs`.

it behaves like this:

- if `--force` is not set, try reading `~/.hpc-login-cache.json`
- if a non-expired token exists for the same username, try downloading the private key with it
- if cached token download succeeds, stop there
- if cached token download fails, fall back to a fresh login
- after fresh login succeeds, download the key and then rewrite the cache

this is the main reliability improvement in the current codebase: expired or rejected cached tokens no longer look like successful runs.

### 3. cas login page request

the login flow begins in `login()` in `src/login.rs`.

first request:

- `GET https://hpc.cugb.edu.cn/sso/login?...`
- includes browser-like headers such as `User-Agent`, `Accept`, and `Accept-Language`
- redirect following is disabled in `ureq`

goals of this step:

- collect initial cookies from `set-cookie`
- parse the login html
- extract the hidden `execution` token required by the cas form

if the html no longer contains the expected `execution` field, login aborts with `Failed to get execution token`.

### 4. password encryption

the plaintext password is never posted directly.

`src/crypto.rs` does this:

- fetch upstream `login.js`
- extract the rsa public key from `var key = '...'`
- if fetch or extraction fails, fall back to a baked-in default public key
- convert the base64 public key into pem format
- encrypt the password with rsa pkcs1 v1.5
- base64-encode the encrypted bytes for form submission

if the upstream key is malformed, encryption now returns an error instead of panicking.

### 5. cas form submission

the second major request is the cas login post:

- method: `POST`
- content type: `application/x-www-form-urlencoded`
- fields include:
  - `username`
  - encrypted `password`
  - `execution`
  - `_eventId=submit`
  - `submit=登录`

after submission, the code handles two success-shaped branches:

- `302` redirect flow with `ticket=...`
- `200` html response with javascript redirect

anything else is treated as login failure.

### 6. ticket exchange and redirect handling

for the redirect-based branch, `src/login.rs` follows the sso handoff manually.

important details:

- cookies are accumulated in a `HashMap<String, String>`
- outgoing `Cookie` header is built by `get_cookie_string()`
- relative redirect locations are normalized against `https://hpc.cugb.edu.cn`
- the `ticket` query param is extracted from the redirect target

manual redirect handling is necessary because the earlier automatic behavior was not reliable for this auth chain.

### 7. jwt retrieval

once the cas session is established, the tool requests:

`https://hpc.cugb.edu.cn/ac/api/user/getCurrentUserInfo.action?includeToken=true&refresh=true`

success criteria:

- response code is http success
- json field `code == "0"`
- `data.tokenList` is not empty

the first token in `tokenList` is used as the jwt-like bearer token for the next step.

### 8. private key download

the private key is downloaded from:

`https://gridview.cugb.edu.cn:6081/sothisai/api/eshell/action/downloadkey`

request characteristics:

- method: `GET`
- custom header: `token: <jwt>`
- `Accept: application/json`

response validation is strict:

- only `code == "0"` with `data: Some(...)` is treated as success
- any other shape becomes an error

the private key payload is written to:

- `~/.hpckey`

on unix systems the file mode is then forced to `0600`.

### 9. token cache write

after a successful fresh login and key download, `src/cache.rs` rewrites:

- `~/.hpc-login-cache.json`

stored fields:

- `username`
- `token`
- `expires_at`
- `created_at`

the cache ttl is currently hardcoded to:

- `2 * 60 * 60 * 1000`
- effectively 2 hours

the file mode is also tightened to `0600` on unix systems.

## data and control boundaries

### credential boundary

plaintext credentials only live in process memory.

- `main.rs` gathers them
- `login.rs` uses them to submit the cas form
- they are not cached to disk by the rust cli

### token boundary

the cache file stores the token because it is used to skip re-login for a short window.

that means token handling is split:

- memory: login flow and immediate download
- disk: local cache file for reuse

### key boundary

the ssh private key is the final persistent artifact.

- it is fetched from the remote api
- written to `~/.hpckey`
- permissions are reduced to owner-only on unix

## current known mismatches

the architecture is more reliable than some of the top-level metadata currently suggests.

examples:

- `Cargo.toml` package version is newer than the clap-declared version string in `src/cli.rs`
- README usage is intentionally simple, but the real behavior around `--quiet`, `--verbose`, and cache fallback is richer than the short summary there

that is why the dedicated docs in this directory exist.
