# Widget RS - 一键打包 Windows 安装程序
# 使用方法：在项目根目录执行 .\build-installer.ps1

param(
    [string]$Version = "0.1.0",
    [switch]$SkipBuild  # 跳过 Rust 编译（已有 release 版本时使用）
)

$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot

Write-Host "=== Widget RS 打包脚本 ===" -ForegroundColor Cyan
Write-Host "版本: $Version" -ForegroundColor Gray

# ─── 1. 编译 Release 版本 ──────────────────────────────────────────────
if (-not $SkipBuild) {
    Write-Host "`n[1/3] 编译 Release 版本..." -ForegroundColor Yellow
    Push-Location $Root
    
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "编译失败！" -ForegroundColor Red
        exit 1
    }
    
    Pop-Location
    Write-Host "    编译完成" -ForegroundColor Green
} else {
    Write-Host "`n[1/3] 跳过编译（使用已有 Release 版本）" -ForegroundColor Gray
}

# 检查可执行文件是否存在
$ExePath = Join-Path $Root "target\release\widget-rs.exe"
if (-not (Test-Path $ExePath)) {
    Write-Host "错误：找不到 $ExePath" -ForegroundColor Red
    Write-Host "请先运行 cargo build --release" -ForegroundColor Red
    exit 1
}

$ExeSize = [math]::Round((Get-Item $ExePath).Length / 1MB, 2)
Write-Host "    可执行文件: widget-rs.exe ($ExeSize MB)" -ForegroundColor Gray

# ─── 2. 创建输出目录 ───────────────────────────────────────────────────
Write-Host "`n[2/3] 准备输出目录..." -ForegroundColor Yellow

$InstallerDir = Join-Path $Root "installer"
if (-not (Test-Path $InstallerDir)) {
    New-Item -ItemType Directory -Path $InstallerDir | Out-Null
}

# ─── 3. 运行 Inno Setup ───────────────────────────────────────────────
Write-Host "`n[3/3] 生成安装程序..." -ForegroundColor Yellow

# 查找 Inno Setup 编译器
$InnoSetupPaths = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 5\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 5\ISCC.exe",
    "D:\tools\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe"
)

$ISCC = $null
foreach ($path in $InnoSetupPaths) {
    if (Test-Path $path) {
        $ISCC = $path
        break
    }
}

if ($null -eq $ISCC) {
    Write-Host ""
    Write-Host "未找到 Inno Setup！请先安装：" -ForegroundColor Red
    Write-Host "  https://jrsoftware.org/isdl.php" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "安装后重新运行此脚本，或手动用 Inno Setup Compiler 打开 widget-rs.iss" -ForegroundColor Yellow
    
    # 尝试打开浏览器下载页面
    $answer = Read-Host "是否打开 Inno Setup 下载页面？(Y/N)"
    if ($answer -eq "Y" -or $answer -eq "y") {
        Start-Process "https://jrsoftware.org/isdl.php"
    }
    exit 1
}

Write-Host "    Inno Setup: $ISCC" -ForegroundColor Gray

$IssFile = Join-Path $Root "widget-rs.iss"
& $ISCC $IssFile

if ($LASTEXITCODE -ne 0) {
    Write-Host "Inno Setup 编译失败！" -ForegroundColor Red
    exit 1
}

# ─── 完成 ─────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "=== 打包完成！===" -ForegroundColor Green

$InstallerFile = Get-ChildItem -Path $InstallerDir -Filter "Widget-RS-Setup-*.exe" |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1

if ($InstallerFile) {
    $InstallerSize = [math]::Round($InstallerFile.Length / 1MB, 2)
    Write-Host "安装包: $($InstallerFile.FullName)" -ForegroundColor Cyan
    Write-Host "大小:   $InstallerSize MB" -ForegroundColor Gray
    
    # 询问是否立即运行安装程序
    $answer = Read-Host "`n是否立即运行安装程序测试？(Y/N)"
    if ($answer -eq "Y" -or $answer -eq "y") {
        Start-Process $InstallerFile.FullName
    }
}
