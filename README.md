[简体中文](./README_ZH.md)

# OSS Share

OSS Share is a Windows 11 tray app for fast file sharing through Alibaba Cloud OSS. It integrates with the Windows Explorer context menu, uploads files to OSS, generates a presigned link, copies that link to the clipboard, and keeps a lightweight file management UI for follow-up actions.

## Features

- Windows 11 Explorer context menu upload
- Tray-based desktop app built with Tauri 2
- Immediate upload from the app window by drag and drop
- Upload status feedback and recent file management
- Presigned share links copied to the clipboard
- Local configuration with DPAPI-encrypted secret storage

## Architecture

This repository contains three Windows-specific parts:

- `src-tauri/`: the Tauri desktop app and Rust backend
- `src/`: the Vue 3 + TypeScript frontend rendered inside the Tauri window
- `shell-extension/`: a Rust COM DLL that adds the Windows 11 context-menu command
- `sparse-package/`: the sparse package manifest and assets required for the packaged shell extension
- `installer/nsis/`: the NSIS installer script

The shell extension sends file paths to the running app through a named pipe. The app uploads the file, generates a presigned OSS URL, copies it to the clipboard, and surfaces status in the UI and notifications.

## Requirements

- Windows 11 x64, version `10.0.22000.0` or newer
- Node.js 20+
- Rust stable toolchain with the MSVC target
- Visual Studio Build Tools with C++ desktop tooling
- WebView2 Runtime
- NSIS, if you want to build the installer
- Windows SDK tools (`makeappx.exe`, `signtool.exe`) if you want to rebuild the sparse package

## Local Development

Install dependencies:

```powershell
npm install
```

Run the desktop app in development mode:

```powershell
npm run tauri dev
```

Notes:

- The app starts as a tray app. Open the main window from the tray menu.
- `npm run dev` only starts the Vite frontend.
- Explorer context-menu integration is not available from plain frontend dev mode. That path depends on the packaged shell extension flow described below.

Build the web frontend only:

```powershell
npm run build
```

Build the shell extension DLL:

```powershell
npm run build:shell
```

Build the Tauri release app:

```powershell
npm run build:app
```

Build both release binaries:

```powershell
npm run build:all
```

## Configuration

At runtime the app stores its configuration at:

```text
%APPDATA%\oss-share\config.toml
```

`access_key_secret` is encrypted with Windows DPAPI before being persisted.

## Building a Local Installer

The repository intentionally does not ship any signing certificate, `.pfx`, `.cer`, `.msix`, or built installer `.exe`. If you want a working installer, generate and sign those artifacts locally.

### 1. Build the app binaries

```powershell
npm install
npm run build:shell
npm run build:app
```

This should produce:

- `src-tauri\target\release\oss-share.exe`
- `shell-extension\target\release\oss_share_shell.dll`

### 2. Generate a development certificate

The sparse package manifest currently uses publisher `CN=OSSShare`, so the certificate subject must match unless you also update `sparse-package/AppxManifest.xml`.

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

### 3. Pack and sign the sparse package

Run these commands from a Developer PowerShell where `makeappx.exe` and `signtool.exe` are available:

```powershell
New-Item -ItemType Directory -Force .\artifacts | Out-Null
makeappx.exe pack /d .\sparse-package /p .\artifacts\OSSShare.msix /o /nv
signtool.exe sign /fd SHA256 /f .\OSSShare.pfx /p changeit .\artifacts\OSSShare.msix
Copy-Item .\artifacts\OSSShare.msix .\sparse-package\OSSShare.msix -Force
```

`/nv` is required here because this is a sparse package that resolves `oss-share.exe` and `oss_share_shell.dll` from the external install location instead of embedding them into the MSIX itself.

### 4. Build the NSIS installer

```powershell
makensis installer\nsis\installer.nsi
```

That script expects these local files to exist:

- `src-tauri\target\release\oss-share.exe`
- `shell-extension\target\release\oss_share_shell.dll`
- `sparse-package\OSSShare.msix`
- `OSSShare.cer`

The resulting installer is written to `installer\nsis\OSSShare-Setup.exe`.

## Repository Notes

- `docs/superpowers/` is intentionally kept. It contains the original specs and implementation plans as part of the project's Vibe Coding history.
- This project is Windows-only by design.
- If you change the package publisher or app identity, regenerate the certificate and update the sparse package manifest accordingly.

## GitHub Releases

The repository includes a tag-driven GitHub Actions workflow at `.github/workflows/release.yml`.

Before using it, configure these repository secrets:

- `WINDOWS_CERT_PFX_BASE64`: base64-encoded contents of your signing `.pfx`
- `WINDOWS_CERT_PASSWORD`: password for that `.pfx`

PowerShell example for generating the base64 payload locally:

```powershell
[Convert]::ToBase64String(
  [IO.File]::ReadAllBytes((Resolve-Path .\OSSShare.pfx))
)
```

Release flow:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

The workflow will:

- build and sign `oss-share.exe` and `oss_share_shell.dll`
- build and sign the sparse package
- build and sign the NSIS installer
- create or update the GitHub Release for the pushed tag
- upload a signed installer, supplementary zip bundles, and `SHA256SUMS.txt`

Tags containing a hyphen, such as `v0.2.0-beta.1`, are published as prereleases.

## License

MIT. See [LICENSE](./LICENSE).
