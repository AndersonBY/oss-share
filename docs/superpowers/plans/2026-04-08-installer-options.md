# Installer Options Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an install-time desktop shortcut option and a finish-page run-app option to the NSIS installer.

**Architecture:** Use a small `nsDialogs` custom page before installation to capture whether a desktop shortcut should be created, then use the standard MUI finish-page run checkbox to optionally launch the app. Keep the start menu shortcut always present and launch the app through `explorer.exe` so the installed app runs in the normal user context rather than elevated installer context.

**Tech Stack:** NSIS, MUI2, nsDialogs

---

### Task 1: Add installer option state

**Files:**
- Modify: `installer/nsis/installer.nsi`
- Test: `installer/nsis/installer.nsi`

- [ ] **Step 1: Record the missing behaviors as the failing target**

```text
Current installer lacks:
1. A pre-install checkbox for desktop shortcut creation.
2. A finish-page checkbox to optionally run the app.
```

- [ ] **Step 2: Run build to verify current script is the old behavior**

Run: `.tools\nsis-3.10\makensis.exe installer\nsis\installer.nsi`
Expected: PASS, but installer UI still has no new option page or finish-page run checkbox.

- [ ] **Step 3: Write minimal implementation**

```nsis
!include "nsDialogs.nsh"
!include "LogicLib.nsh"
!include "WinMessages.nsh"

Var CreateDesktopShortcut
Var DesktopShortcutCheckbox

Page custom InstallerOptionsPageCreate InstallerOptionsPageLeave
```

Add a custom page that defaults the desktop shortcut checkbox to unchecked and stores the result in `$CreateDesktopShortcut`.

- [ ] **Step 4: Run build to verify syntax passes**

Run: `.tools\nsis-3.10\makensis.exe installer\nsis\installer.nsi`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add installer/nsis/installer.nsi
git commit -m "feat: add installer desktop shortcut option"
```

### Task 2: Add finish-page run option

**Files:**
- Modify: `installer/nsis/installer.nsi`
- Test: `installer/nsis/installer.nsi`

- [ ] **Step 1: Define finish-page run checkbox**

```nsis
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_TEXT "运行 OSS Share"
!define MUI_FINISHPAGE_RUN_FUNCTION LaunchInstalledApp
```

- [ ] **Step 2: Launch app from unelevated shell path**

```nsis
Function LaunchInstalledApp
    Exec '"$WINDIR\explorer.exe" "$INSTDIR\oss-share.exe"'
FunctionEnd
```

This keeps the installer elevated while starting the app through the logged-in user's shell context.

- [ ] **Step 3: Create desktop shortcut only when selected**

```nsis
${If} $CreateDesktopShortcut == ${BST_CHECKED}
    CreateShortcut "$DESKTOP\OSS Share.lnk" "$INSTDIR\oss-share.exe"
${EndIf}
```

- [ ] **Step 4: Remove the desktop shortcut on uninstall**

```nsis
Delete "$DESKTOP\OSS Share.lnk"
```

- [ ] **Step 5: Run build to verify syntax passes**

Run: `.tools\nsis-3.10\makensis.exe installer\nsis\installer.nsi`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add installer/nsis/installer.nsi
git commit -m "feat: add installer finish-page run option"
```

### Task 3: Verify packaged installer output

**Files:**
- Modify: `installer/nsis/installer.nsi`

- [ ] **Step 1: Rebuild app artifacts**

Run: `npx tauri build`
Expected: PASS.

- [ ] **Step 2: Rebuild the signed sparse package**

Run: `makeappx.exe pack ...` then `signtool.exe sign ...`
Expected: PASS.

- [ ] **Step 3: Rebuild the installer**

Run: `.tools\nsis-3.10\makensis.exe installer\nsis\installer.nsi`
Expected: PASS and output `installer\nsis\OSSShare-Setup.exe`.

- [ ] **Step 4: Manual validation checklist**

```text
1. Installer shows a pre-install desktop shortcut checkbox, unchecked by default.
2. Installer still creates the start menu shortcut.
3. Finish page shows a checked "运行 OSS Share" option.
4. If checked, the app launches after install in normal user context.
5. If desktop shortcut was selected, the desktop icon exists after install.
```

- [ ] **Step 5: Commit**

```bash
git add installer/nsis/installer.nsi installer/nsis/OSSShare-Setup.exe
git commit -m "build: refresh installer with new options"
```
