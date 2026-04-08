Unicode true
!include "MUI2.nsh"
!include "nsDialogs.nsh"
!include "LogicLib.nsh"
!include "WinMessages.nsh"

Name "OSS Share"
OutFile "OSSShare-Setup.exe"
InstallDir "$LOCALAPPDATA\OSSShare"
RequestExecutionLevel admin

!define MUI_FINISHPAGE_TEXT "OSS Share 已安装完成。你可以选择立即运行应用。应用会以当前登录用户身份启动，以确保右键上传正常工作。"
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_TEXT "运行 OSS Share"
!define MUI_FINISHPAGE_RUN_FUNCTION LaunchInstalledApp

!define MUI_ICON "..\..\src-tauri\icons\icon.ico"
!define MUI_UNICON "..\..\src-tauri\icons\icon.ico"

Var CreateDesktopShortcut
Var DesktopShortcutCheckbox

!insertmacro MUI_PAGE_DIRECTORY
Page custom InstallerOptionsPageCreate InstallerOptionsPageLeave
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "SimpChinese"

Function .onInit
    StrCpy $CreateDesktopShortcut ${BST_UNCHECKED}
FunctionEnd

Function InstallerOptionsPageCreate
    !insertmacro MUI_HEADER_TEXT "安装选项" "选择是否创建桌面快捷方式"

    nsDialogs::Create 1018
    Pop $0

    ${If} $0 == error
        Abort
    ${EndIf}

    ${NSD_CreateLabel} 0 0 100% 12u "请选择是否在桌面创建快捷方式。"
    Pop $0

    ${NSD_CreateCheckbox} 0 22u 100% 12u "创建桌面快捷方式"
    Pop $DesktopShortcutCheckbox
    ${NSD_SetState} $DesktopShortcutCheckbox $CreateDesktopShortcut

    nsDialogs::Show
FunctionEnd

Function InstallerOptionsPageLeave
    ${NSD_GetState} $DesktopShortcutCheckbox $CreateDesktopShortcut
FunctionEnd

Function LaunchInstalledApp
    Exec '"$WINDIR\explorer.exe" "$INSTDIR\oss-share.exe"'
FunctionEnd

Section "Install"
    SetOutPath $INSTDIR

    ; Kill running instance
    nsExec::ExecToLog 'taskkill /f /im oss-share.exe'

    ; Clean up old shell\command registration from previous versions
    DeleteRegKey HKCU "SOFTWARE\Classes\*\shell\OSSShare\command"
    DeleteRegKey HKCU "SOFTWARE\Classes\*\shell\OSSShare"
    DeleteRegKey HKCU "SOFTWARE\Classes\CLSID\{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}"

    ; Copy files
    File "..\..\src-tauri\target\release\oss-share.exe"
    File "..\..\shell-extension\target\release\oss_share_shell.dll"
    File "..\..\sparse-package\OSSShare.msix"
    File "..\..\OSSShare.cer"

    ; Install certificate to Trusted Root (requires admin)
    nsExec::ExecToLog 'certutil -addstore Root "$INSTDIR\OSSShare.cer"'

    ; Remove old sparse package if exists
    nsExec::ExecToLog 'powershell -NoProfile -Command "Get-AppxPackage -Name OSSShare 2>$null | Remove-AppxPackage -ErrorAction SilentlyContinue"'

    ; Register sparse package with external location
    nsExec::ExecToLog 'powershell -NoProfile -Command "Add-AppxPackage -Path \"$INSTDIR\OSSShare.msix\" -ExternalLocation \"$INSTDIR\""'

    ; Write install path to registry
    WriteRegStr HKCU "SOFTWARE\OSSShare" "InstallPath" "$INSTDIR"

    ; Start Menu shortcut (launches as the current logged-in user later)
    CreateDirectory "$SMPROGRAMS\OSS Share"
    CreateShortcut "$SMPROGRAMS\OSS Share\OSS Share.lnk" "$INSTDIR\oss-share.exe"
    ${If} $CreateDesktopShortcut == ${BST_CHECKED}
        CreateShortcut "$DESKTOP\OSS Share.lnk" "$INSTDIR\oss-share.exe"
    ${EndIf}

    ; Auto-start on login
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" \
        "OSSShare" '"$INSTDIR\oss-share.exe"'

    ; Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Add/Remove Programs entry
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare" \
        "DisplayName" "OSS Share"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare" \
        "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare" \
        "DisplayIcon" '"$INSTDIR\oss-share.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare" \
        "Publisher" "OSS Share"

    ; Notify shell of changes
    System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
SectionEnd

Section "Uninstall"
    ; Kill running instance
    nsExec::ExecToLog 'taskkill /f /im oss-share.exe'

    ; Remove sparse package
    nsExec::ExecToLog 'powershell -NoProfile -Command "Get-AppxPackage -Name OSSShare 2>$null | Remove-AppxPackage -ErrorAction SilentlyContinue"'

    ; Remove certificate
    nsExec::ExecToLog 'certutil -delstore Root "CN=OSSShare"'

    ; Remove files
    Delete "$INSTDIR\oss-share.exe"
    Delete "$INSTDIR\oss_share_shell.dll"
    Delete "$INSTDIR\OSSShare.msix"
    Delete "$INSTDIR\OSSShare.cer"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    Delete "$DESKTOP\OSS Share.lnk"
    Delete "$SMPROGRAMS\OSS Share\OSS Share.lnk"
    RMDir "$SMPROGRAMS\OSS Share"

    ; Clean registry
    DeleteRegKey HKCU "SOFTWARE\OSSShare"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "OSSShare"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OSSShare"

    ; Notify shell of changes
    System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
SectionEnd
