# ghpc

[English](README.md)

`ghpc` 是一个用于 cugb hpc 登录自动化的 rust 命令行工具。它会通过 cas sso 登录，在可用时复用本地短期 token cache，并把 ssh 私钥下载到 `~/.hpckey`。

如果你经常用 cugb hpc，这个工具的目的只有一个：把一套烦人的网页操作，变成一条命令。

## 为什么写这个工具

学校的 hpc 平台不是不能用，只是体验不太舒服。

- 网页有点慢
- 浏览器里的 eshell 不太想长期依赖
- ssh key 有效期短
- 刷新流程重复，而且很快就会烦

每次 key 过期，通常都要：

1. 打开门户
2. 等页面加载
3. 一路点进 hpc 页面
4. 再找到下载入口
5. 把 key 保存到本地

这事不算难，但足够重复，值得自动化。

所以这个项目就做一件事：把流程收起来。

- 登录
- cache 有效时直接复用
- cache 失效时自动重登
- 把 key 写到 `~/.hpckey`

## 安装

```bash
git clone https://github.com/littlepenguin66/cugb-hpc-getkey.git
cd cugb-hpc-getkey
cargo build --release
```

编译后的二进制文件在：

```bash
target/release/ghpc
```

如果你想全局使用：

```bash
mv target/release/ghpc ~/.local/bin/
```

## 快速开始

交互式使用：

```bash
ghpc
```

使用环境变量：

```bash
export HPC_USERNAME=your_username
export HPC_PASSWORD=your_password
ghpc
```

强制重新登录：

```bash
ghpc --force
```

查看 cache 状态：

```bash
ghpc --status
```

调试失败流程：

```bash
ghpc --force --verbose
```

## 选项

| 参数 | 说明 |
|------|------|
| `-u, --username` | HPC 用户名，或使用 `HPC_USERNAME` |
| `-p, --password` | HPC 密码，或使用 `HPC_PASSWORD` |
| `-f, --force` | 跳过 cache，执行一次全新登录 |
| `-s, --status` | 只输出 cache 状态 |
| `-q, --quiet` | 抑制信息输出 |
| `-v, --verbose` | 输出调试日志和 token |

## 它实际做了什么

1. 请求 cugb hpc 的 cas 登录页
2. 提取 `execution` token
3. 用上游 rsa 公钥加密密码
4. 手动完成 sso 跳转链路
5. 从 hpc api 获取 jwt token
6. 从 gridview 下载 ssh 私钥
7. 把私钥写到 `~/.hpckey`
8. 把短期 cache 写到 `~/.hpc-login-cache.json`

如果 cache token 下载失败，`ghpc` 会自动回退到一次全新登录。

## 文档

核心文档：

- [architecture](docs/architecture.md)
- [cli](docs/cli.md)
- [security](docs/security.md)
- [troubleshooting](docs/troubleshooting.md)

release notes：

- [v2026.3.18](docs/release/v2026.3.18.md)
- [v2026.4.11](docs/release/v2026.4.11.md)

## 配合 AI 使用

仓库里也带了一套面向 ai 助手的 skill：

```text
.skills/cugb-hpc-getkey/
```

它主要是帮助 coding assistant 理解登录流程和本地文件布局。

安全提醒：

- 不要随便把真实 hpc 密码或私钥发给 ai 系统
- 更推荐在本地运行工具，只把配置或排障问题交给 ai

## 备注

- 上游 rsa 公钥以后可能还会变
- 如果登录突然坏了，先看 `docs/troubleshooting.md`
- 如果你在意凭据安全，建议顺手看一下 `docs/security.md`

## 许可证

参见 [LICENSE](LICENSE)。
