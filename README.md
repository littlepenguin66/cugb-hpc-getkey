# ghpc

CUGB HPC auto-login tool - authenticates and downloads your HPC private key.

## Why I Made This

The school's HPC platform sucks. The web UI is laggy and awful to use. The eshell in browser is so slow it's barely usable.

They let you download an SSH key, but for security reasons it only lasts 12 hours. After that, you have to:

1. Log in to the web portal
2. Wait for the laggy page to load
3. Head to the eshell page
4. Find the download button
5. Download the key (with some messy-ass name) to your Downloads folder

And you have to do this every 12 hours. Annoying as hell.

So I made this tiny tool to make my life easier. Now I just run `ghpc` and done. It handles the login, caches your token, and grabs the key - all automatically.

## Installation

```bash
git clone https://github.com/littlepenguin66/cugb-hpc-getkey.git
cd cugb-hpc-getkey
cargo build --release
```

The binary will be at `target/release/ghpc`.

For convenience, move it to your local bin:

```bash
mv target/release/ghpc ~/.local/bin/
```

Then you can run `ghpc` from anywhere.

## Usage

```bash
# Interactive prompt (will ask for username/password)
./target/release/ghpc

# With credentials
./target/release/ghpc -u <username> -p <password>

# Force re-login (ignore cached token)
./target/release/ghpc --force

# Check cache status
./target/release/ghpc --status
```

### Options

| Flag | Description |
|------|-------------|
| `-u, --username` | HPC username (or set `HPC_USERNAME` env var) |
| `-p, --password` | HPC password (or set `HPC_PASSWORD` env var) |
| `-f, --force` | Force re-login, ignore cached token |
| `-s, --status` | Show cache status |
| `-q, --quiet` | Suppress info output |
| `-v, --verbose` | Enable debug output |

## How It Works

1. Authenticates to CUGB HPC via CAS SSO
2. Caches JWT token for 2 hours (`~/.hpc-login-cache.json`)
3. Downloads private key to `~/.hpckey`

Subsequent runs use the cached token until it expires.

## Use with AI

In the AI age, you can directly ask AI to help with HPC tasks. This project includes a skill file (`.skills/cugb-hpc-getkey/`) that helps AI assistants understand how to work with the HPC system.

**Security Warning**: Never share your HPC credentials (username/password) with AI assistants unless you fully trust them. This tool runs locally and keeps your credentials secure on your machine.

## Notes

The school's RSA public key may change over time. If login fails, try updating to the latest version of this tool.

If you encounter any issues, feel free to open an issue on GitHub.

## License

See [LICENSE](LICENSE).
