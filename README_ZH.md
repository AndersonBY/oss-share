[English](./README.md)

# OSS Share

OSS Share 是一个面向 Windows 11 的阿里云 OSS 文件分享工具。它常驻系统托盘，支持资源管理器右键上传、窗口内拖拽上传、生成预签名链接并自动复制到剪贴板，同时提供一个轻量的文件管理界面用于复制链接和删除文件。

## 功能

- Windows 11 一级右键菜单上传
- 基于 Tauri 2 的托盘常驻桌面应用
- 支持将文件直接拖到窗口任意位置后立即上传
- 上传状态反馈与文件管理列表
- 自动复制预签名分享链接到剪贴板
- 本地配置持久化，密钥使用 Windows DPAPI 加密保存

## 架构

仓库主要由 3 个 Windows 相关部分组成：

- `src-tauri/`：Tauri 桌面应用和 Rust 后端
- `src/`：运行在 Tauri 窗口中的 Vue 3 + TypeScript 前端
- `shell-extension/`：给 Windows 11 右键菜单提供入口的 Rust COM DLL
- `sparse-package/`：Shell Extension 所需的 sparse package 清单和资源
- `installer/nsis/`：NSIS 安装脚本

Shell Extension 通过 Named Pipe 把文件路径发给主程序。主程序完成上传、生成预签名链接、复制链接到剪贴板，并在 UI 和系统通知里反馈状态。

## 环境要求

- Windows 11 x64，版本不低于 `10.0.22000.0`
- Node.js 20+
- Rust stable toolchain，且使用 MSVC 目标
- Visual Studio Build Tools，安装桌面 C++ 工具链
- WebView2 Runtime
- 需要构建安装包时安装 NSIS
- 需要重建 sparse package 时保证 Windows SDK 的 `makeappx.exe`、`signtool.exe` 可用

## 本地开发

先安装依赖：

```powershell
npm install
```

启动桌面开发模式：

```powershell
npm run tauri dev
```

说明：

- 应用默认以托盘程序方式启动，需要从托盘菜单打开主窗口。
- `npm run dev` 只会启动 Vite 前端开发服务器。
- 资源管理器右键菜单链路不属于纯前端开发模式，它依赖下面的打包和安装流程。

只构建前端：

```powershell
npm run build
```

构建 Shell Extension DLL：

```powershell
npm run build:shell
```

构建 Tauri 发布版主程序：

```powershell
npm run build:app
```

一起构建两个发布版二进制：

```powershell
npm run build:all
```

## 配置文件

运行后配置会写入：

```text
%APPDATA%\oss-share\config.toml
```

其中 `access_key_secret` 在写盘前会先经过 Windows DPAPI 加密。

## 本地构建安装程序

仓库现在不会再提交任何签名证书、`.pfx`、`.cer`、`.msix` 或现成安装包 `.exe`。如果你要自己打一个可安装版本，需要在本地生成并签名这些产物。

### 1. 先构建应用二进制

```powershell
npm install
npm run build:shell
npm run build:app
```

完成后应能看到：

- `src-tauri\target\release\oss-share.exe`
- `shell-extension\target\release\oss_share_shell.dll`

### 2. 生成开发用证书

当前 sparse package 的发布者是 `CN=OSSShare`，所以证书主题必须一致；如果你改了 `sparse-package/AppxManifest.xml` 里的 Publisher，这一步也要跟着调整。

```powershell
$cert = New-SelfSignedCertificate `
  -Type Custom `
  -Subject "CN=OSSShare" `
  -KeyAlgorithm RSA `
  -KeyLength 2048 `
  -HashAlgorithm SHA256 `
  -CertStoreLocation "Cert:\CurrentUser\My"

$password = ConvertTo-SecureString "changeit" -AsPlainText -Force

Export-PfxCertificate -Cert $cert -FilePath .\OSSShare.pfx -Password $password
Export-Certificate -Cert $cert -FilePath .\OSSShare.cer
```

### 3. 打包并签名 sparse package

请在带有 Windows SDK 工具链的 Developer PowerShell 中执行：

```powershell
makeappx.exe pack /d .\sparse-package /p .\sparse-package\OSSShare.msix /o
signtool.exe sign /fd SHA256 /f .\OSSShare.pfx /p changeit .\sparse-package\OSSShare.msix
```

### 4. 构建 NSIS 安装器

```powershell
makensis installer\nsis\installer.nsi
```

这个脚本会读取以下本地产物：

- `src-tauri\target\release\oss-share.exe`
- `shell-extension\target\release\oss_share_shell.dll`
- `sparse-package\OSSShare.msix`
- `OSSShare.cer`

最终安装器输出到 `installer\nsis\OSSShare-Setup.exe`。

## 仓库说明

- `docs/superpowers/` 被刻意保留，用来保存这个项目的 specs / plans，也作为 Vibe Coding 过程记录的一部分。
- 这是一个明确的 Windows-only 项目。
- 如果你修改了包身份或发布者，请同步更新 manifest，并重新生成对应证书。

## GitHub 自动发布

仓库已经包含一个基于 tag 触发的 GitHub Actions 工作流：`.github/workflows/release.yml`。

使用前需要先在仓库 Secrets 中配置：

- `WINDOWS_CERT_PFX_BASE64`：签名 `.pfx` 文件内容的 base64 字符串
- `WINDOWS_CERT_PASSWORD`：该 `.pfx` 的密码

本地把 `PFX` 转成 base64 的 PowerShell 示例：

```powershell
[Convert]::ToBase64String(
  [IO.File]::ReadAllBytes((Resolve-Path .\OSSShare.pfx))
)
```

发布方式：

```powershell
git tag v0.1.0
git push origin v0.1.0
```

这个工作流会自动：

- 构建并签名 `oss-share.exe` 和 `oss_share_shell.dll`
- 构建并签名 sparse package
- 构建并签名 NSIS 安装器
- 为对应 tag 创建或更新 GitHub Release
- 上传签名后的安装器、补充 zip 包和 `SHA256SUMS.txt`

带有连字符的 tag，例如 `v0.2.0-beta.1`，会自动作为 prerelease 发布。

## 许可证

MIT，见 [LICENSE](./LICENSE)。
