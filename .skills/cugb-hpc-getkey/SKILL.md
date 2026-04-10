---
name: cugb-hpc-getkey
description: How to help users with CUGB HPC login and SSH key retrieval. Use this skill whenever the user mentions HPC login, CUGB authentication, hpc.cugb.edu.cn, gridview, downloading SSH keys for HPC cluster access, or needs to automate login to the China University of Geosciences Beijing HPC system. This includes troubleshooting login issues, setting up the tool, checking cache status, or understanding the authentication flow.
---

# CUGB HPC Login Helper

This skill helps you assist users with HPC (High Performance Computing) login automation for China University of Geosciences Beijing (CUGB).

## When to Use This Skill

Use this skill when the user:
- Wants to log in to HPC cluster (`hpc.cugb.edu.cn`)
- Needs to download SSH private key (`~/.hpckey`)
- Asks about getting JWT token for gridview access
- Has login issues or errors
- Wants to check cache status
- Asks to force refresh token

## Quick Start

### Run Login Tool

```bash
cd .skills/cugb-hpc-getkey
bun install
export HPC_USERNAME=your_username
export HPC_PASSWORD=your_password
bun run index.ts

# Or use command line arguments
bun run index.ts -u <username> -p <password>
```

### Available Options

| Option | Short | Description |
|--------|-------|-------------|
| `--username` | `-u` | Username |
| `--password` | `-p` | Password |
| `--quiet` | `-q` | Suppress info output and print only token |
| `--verbose` | `-v` | Enable verbose logging and print token |
| `--force` | `-f` | Force refresh, ignore cache |
| `--status` | `-s` | Check cache status |

### Check Cache Status

```bash
bun run index.ts --status
```

Example output:
```
Cache status: Valid
Username: username
Expires at: 2026/3/18 21:34:18
```

### Force Refresh Token

```bash
bun run index.ts --force
# or
bun run index.ts -u <username> -p <password> -f
```

## Skill Structure

```
.skills/cugb-hpc-getkey/
├── index.ts      # Entry point
├── cli.ts        # CLI argument parsing
├── login.ts      # Login flow
├── crypto.ts     # RSA encryption
├── session.ts    # Cookie handling
├── cache.ts      # Token caching
├── types.ts      # Type definitions
└── package.json
```

## Authentication Flow (6 Steps)

1. **Get Login Page** - Fetch CAS login page, extract `execution` token
2. **Encrypt Password** - Encrypt password using RSA public key
3. **Submit Login** - POST credentials to SSO
4. **Follow Redirect** - Handle CAS ticket exchange for session
5. **Get JWT** - Call API to get token
6. **Download Private Key** - Use JWT to download SSH private key to `~/.hpckey`

If cached token download fails, the tool falls back to a fresh login automatically.

## Troubleshooting

### Error: "Cannot get execution token"

Possible causes:
1. Network issues - Check VPN/network connection
2. SSO page structure changed - Need to update regex matching
3. Cache issues - Clear cache `rm ~/.hpc-login-cache.json`

Diagnostic commands:
```bash
curl -I https://hpc.cugb.edu.cn/sso/login
bun run index.ts -v  # verbose mode
```

### Token Expiration

- Cache validity: 2 hours
- Auto re-login after expiration
- Cached token download failure triggers re-login
- Use `--force` to force refresh

## File Locations

| File | Location |
|------|----------|
| Token Cache | `~/.hpc-login-cache.json` |
| SSH Private Key | `~/.hpckey` (permission 0600) |

## Security Notes

- Prefer `HPC_USERNAME` and `HPC_PASSWORD` environment variables over `-p` on the command line
- Successful default runs save the private key locally and do not print the token
- Use `--quiet` only when you explicitly need the token in stdout
