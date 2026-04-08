# OSS Share — 阿里云 OSS 文件共享工具设计文档

## 概述

OSS Share 是一个 Windows 系统托盘应用，通过阿里云 OSS 实现快速文件共享。用户右键文件选择"共享到 OSS"，程序自动上传并将预签名共享链接复制到剪贴板。

**目标用户：** 个人自用，快速把文件分享给朋友/同事。

**核心体验：** 右键 → 上传 → 链接已在剪贴板，直接粘贴发送。

## 技术栈

- **应用框架：** Rust + Tauri 2.x
- **前端：** Vue 3 + TypeScript
- **Shell Extension：** Rust + windows-rs (COM DLL)
- **安装程序：** NSIS
- **目标平台：** Windows 11

## 架构

两个独立二进制文件：

1. **oss-share.exe** — Tauri 主进程，后台常驻系统托盘，负责上传、配置 UI、文件管理
2. **oss-share-shell.dll** — COM DLL，实现 IExplorerCommand 接口，注册到 Windows 11 新版右键菜单

### 通信方式

Shell DLL 通过 Named Pipe (`\\.\pipe\oss-share`) 将文件路径发送给主进程。DLL 发完即走，不等待响应。主进程处理完成后通过系统通知反馈用户。

### 数据流

```
用户右键文件 → "共享到 OSS"
    ├─→ Shell DLL 获取文件路径
    ├─→ 通过 Named Pipe 发送给主进程
    ├─→ 主进程上传文件到 OSS
    ├─→ 生成预签名 URL
    ├─→ 复制到剪贴板
    └─→ 弹出系统通知 "链接已复制"
```

## 项目结构

```
oss-share/
├── src-tauri/              # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs         # 入口，托盘初始化
│   │   ├── tray.rs         # 系统托盘模块
│   │   ├── oss.rs          # OSS 上传/签名/列表
│   │   ├── ipc.rs          # Named Pipe 服务端
│   │   ├── config.rs       # 配置读写 (config.toml)
│   │   └── commands.rs     # Tauri IPC 命令（前端调用）
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                    # 前端 (Web UI)
│   ├── App.vue             # 主页面（标签页路由）
│   ├── views/
│   │   ├── Settings.vue    # 设置页
│   │   └── Files.vue       # 文件管理页
│   ├── components/
│   │   └── FileRow.vue     # 文件行组件
│   └── main.ts
├── shell-extension/        # 独立 COM DLL 项目
│   ├── src/
│   │   ├── lib.rs          # DLL 入口，COM 注册
│   │   ├── command.rs      # IExplorerCommand 实现
│   │   └── pipe_client.rs  # Named Pipe 客户端
│   └── Cargo.toml
├── installer/              # 安装程序脚本
│   └── nsis/
│       └── installer.nsi
└── package.json
```

## UI 设计

### 系统托盘菜单

右键托盘图标弹出菜单：

- 📂 打开文件管理
- ⚙️ 设置
- 🔗 最近共享（显示最近 3 条记录）
- 退出

### Windows 11 右键菜单

在新版右键菜单一级位置显示"☁️ 共享到 OSS"，支持多选文件逐个上传。

### 主窗口

两个标签页：

**设置页：**
- Access Key ID / Access Key Secret 输入框
- Region 选择（如 oss-cn-hangzhou）
- Bucket 名称
- 上传路径前缀（如 `share/`）
- 链接有效期切换按钮：24h / 7d / 30d
- "测试连接"和"保存"按钮

**文件管理页：**
- 显示配置路径前缀下的文件列表
- 表格列：文件名、大小、上传时间
- 每行操作：复制链接 🔗、删除 🗑
- 顶部搜索框过滤文件名

## 关键依赖

### Tauri 主程序

| 依赖 | 用途 |
|------|------|
| tauri 2.x | 应用框架 |
| tauri-plugin-notification | 系统通知 |
| tauri-plugin-clipboard-manager | 剪贴板操作 |
| aliyun-oss-rust-sdk | OSS API |
| tokio | 异步运行时 |
| serde + toml | 配置序列化 |
| windows-rs | Named Pipe API |

### Shell Extension DLL

| 依赖 | 用途 |
|------|------|
| windows-rs | COM / IExplorerCommand |
| windows-implement | COM 宏辅助 |

DLL 不使用 tokio，保持轻量，使用同步 Named Pipe 写入。

## IPC 协议

Named Pipe 地址：`\\.\pipe\oss-share`

消息格式为 JSON (UTF-8)，以 `\n` 结尾：

```json
{
  "action": "upload",
  "files": ["C:\\Users\\test\\report.pdf", "C:\\Users\\test\\photo.jpg"]
}
```

Shell DLL 为客户端，连接 Pipe → 写入 JSON → 断开。不等待响应。

## 配置文件

路径：`%APPDATA%/oss-share/config.toml`

```toml
[credentials]
access_key_id = "LTAI5t..."
access_key_secret = "DPAPI 加密后的密文"

[oss]
region = "oss-cn-hangzhou"
bucket = "my-share-bucket"
prefix = "share/"

[sharing]
expire_seconds = 604800  # 86400=24h, 604800=7d, 2592000=30d
```

Access Key Secret 使用 Windows DPAPI 加密存储，不明文保存在磁盘上。

## 安装与卸载

使用 NSIS 制作安装程序。

**安装流程：**
1. 释放文件到 `Program Files/OSSShare/`
2. `regsvr32 /s oss-share-shell.dll` — 注册 COM 组件
3. 写入注册表开机自启项
4. 启动 oss-share.exe

**卸载流程：**
1. 关闭 oss-share.exe
2. `regsvr32 /u /s oss-share-shell.dll` — 取消 COM 注册
3. 删除程序文件
4. 清理注册表
5. 可选保留 `%APPDATA%/oss-share/` 配置目录

## 错误处理

| 场景 | 处理方式 |
|------|---------|
| 主进程未运行时右键上传 | DLL 连接 Pipe 失败 → 从注册表 `HKLM\SOFTWARE\OSSShare\InstallPath` 读取路径启动 oss-share.exe → 重试连接 |
| 网络错误 / 上传失败 | 系统通知显示错误信息 |
| 未配置 AK/SK | 首次使用弹出设置窗口引导配置 |
| 大文件上传 (>100MB) | 使用 OSS 分片上传，托盘图标显示进度 |

## 上传通知

上传完成后：
1. 自动将预签名 URL 复制到剪贴板
2. 弹出 Windows 系统通知气泡，显示"文件名 — 链接已复制"

## 共享链接

使用阿里云 OSS 预签名 URL，用户可在设置中选择有效期（24h / 7d / 30d）。文件管理页面可以随时重新生成链接并复制。
