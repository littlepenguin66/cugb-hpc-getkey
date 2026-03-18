# ghpc

CUGB HPC 自动登录工具 - 自动认证并下载 HPC 私钥。

## 为什么开发这个工具

学校的 HPC 平台体验不太好。网页端有点卡顿，使用起来不够顺畅。浏览器里的 eshell 响应也比较慢。

虽说可以下载 SSH 私钥吧，但为了安全起见，密钥有效期只有 12 小时。过期后需要：

1. 登录网页门户
2. 等待页面加载
3. 进入 eshell 页面
4. 找到下载按钮
5. 下载密钥到本地文件夹（文件名也比较复杂）

每 12 小时就要操作这么一次，确实有些麻烦。

所以写了这个小工具来让生活更轻松。现在直接运行 `ghpc` 就行，它会自动处理登录、缓存令牌、获取密钥。

## 安装

```bash
git clone https://github.com/littlepenguin66/cugb-hpc-getkey.git
cd cugb-hpc-getkey
cargo build --release
```

编译后的二进制文件位于 `target/release/ghpc`。

为方便使用，可将其移动到本地 bin 目录：

```bash
mv target/release/ghpc ~/.local/bin/
```

之后即可在任何位置运行 `ghpc`。

## 使用方法

```bash
# 交互式（会提示输入用户名/密码）
./target/release/ghpc

# 提供凭据
./target/release/ghpc -u <用户名> -p <密码>

# 强制重新登录（忽略缓存的令牌）
./target/release/ghpc --force

# 查看缓存状态
./target/release/ghpc --status
```

### 选项

| 参数 | 说明 |
|------|------|
| `-u, --username` | HPC 用户名（或设置 `HPC_USERNAME` 环境变量） |
| `-p, --password` | HPC 密码（或设置 `HPC_PASSWORD` 环境变量） |
| `-f, --force` | 强制重新登录，忽略缓存的令牌 |
| `-s, --status` | 显示缓存状态 |
| `-q, --quiet` | 禁用信息输出 |
| `-v, --verbose` | 启用调试输出 |

## 工作原理

1. 通过 CAS SSO 认证到中国地质大学（北京）HPC 系统
2. 将 JWT 令牌缓存 2 小时（`~/.hpc-login-cache.json`）
3. 下载私钥到 `~/.hpckey`

后续运行会使用缓存的令牌，直到过期。

## 配合 AI 使用

在 AI 时代，可以直接让 AI 帮你操作 HPC。本项目包含一个技能文件（`.skills/cugb-hpc-getkey/`），帮助 AI 助手理解如何与 HPC 系统交互。

**安全警告**：除非完全信任，否则切勿向 AI 助手分享你的 HPC 凭据（用户名/密码）。本工具在本地运行，可以保护你的凭据安全。

## 注意事项

学校的 RSA 公钥可能会定期更换。如果登录失败，请尝试更新到最新版本。

如遇到问题，欢迎在 GitHub 上提 Issue。

## 许可证

参见 [LICENSE](LICENSE)。
