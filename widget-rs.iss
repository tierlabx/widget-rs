; Widget RS - Inno Setup 安装脚本
; 生成 Windows 安装程序 (.exe)
; 
; 使用方法：
;   1. 先运行 build-release.ps1 编译 Release 版本
;   2. 用 Inno Setup Compiler 打开此文件，点击 Build > Compile
;   或直接运行：build-installer.ps1（自动完成以上步骤）

#define MyAppName "Widget RS"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Widget RS"
#define MyAppURL "https://github.com/your-username/widget-rs"
#define MyAppExeName "widget-rs.exe"
#define MyAppDescription "基于 Rust + GPUI 的桌面小部件系统"

[Setup]
; 程序唯一 ID（每个版本应唯一，用 uuidgen 生成）
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
; 默认安装到 Program Files
DefaultDirName={autopf}\{#MyAppName}
; 开始菜单组
DefaultGroupName={#MyAppName}
; 不需要管理员权限（安装到用户目录）
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
; 输出安装包路径和名称
OutputDir=installer
OutputBaseFilename=Widget-RS-Setup-{#MyAppVersion}
; 安装包图标
SetupIconFile=assets\logos\icon.ico
; 压缩算法
Compression=lzma2/ultra64
SolidCompression=yes
; 向导风格
WizardStyle=modern
; 最小 Windows 版本：Win10
MinVersion=10.0
; 显示许可协议（可选）
; LicenseFile=LICENSE

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
; 可选：添加开机启动
Name: "startup"; Description: "开机时自动启动 {#MyAppName}"; \
    GroupDescription: "启动选项:"; Flags: unchecked

[Files]
; 主可执行文件
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; 开始菜单快捷方式
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\卸载 {#MyAppName}"; Filename: "{uninstallexe}"

[Registry]
; 开机启动（仅当用户选择了该任务时）
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; \
    ValueType: string; ValueName: "{#MyAppName}"; \
    ValueData: """{app}\{#MyAppExeName}"""; \
    Flags: uninsdeletevalue; Tasks: startup

[Run]
; 安装完成后启动程序
Filename: "{app}\{#MyAppExeName}"; Description: "立即启动 {#MyAppName}"; \
    Flags: nowait postinstall skipifsilent

[UninstallRun]
; 卸载前先关闭程序
Filename: "taskkill"; Parameters: "/F /IM {#MyAppExeName}"; Flags: runhidden; \
    RunOnceId: "KillApp"

[Code]
// 安装前检查：如果程序正在运行则提示关闭
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
begin
  // 尝试关闭正在运行的实例
  Exec('taskkill', '/F /IM {#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := True;
end;
