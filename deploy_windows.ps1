# Toki 区块链部署脚本 (Windows)
# 用于在 Windows 服务器上自动部署和启动节点

$ErrorActionPreference = "Stop"

# 配置变量
$ServerIP = "182.254.176.30"
$InstanceID = "lhins-i50x"
$DataDir = ".\data"
$LogDir = ".\logs"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Toki 区块链部署脚本 (Windows)" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "服务器信息:" -ForegroundColor Yellow
Write-Host "  IP: $ServerIP" -ForegroundColor White
Write-Host "  实例 ID: $InstanceID" -ForegroundColor White
Write-Host ""

# 检查 Rust 环境
Write-Host "检查 Rust 环境..." -ForegroundColor Green
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust 未安装，正在安装..." -ForegroundColor Red
    Write-Host "请访问 https://rustup.rs/ 安装 Rust" -ForegroundColor Yellow
    exit 1
} else {
    Write-Host "Rust 已安装: $(cargo --version)" -ForegroundColor Green
}

# 创建必要的目录
Write-Host "创建目录..." -ForegroundColor Green
New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

# 编译项目
Write-Host "编译项目..." -ForegroundColor Green
cargo build --release

# 检查启动配置
Write-Host "检查启动配置..." -ForegroundColor Green
if (Test-Path "config_bootstrap.toml") {
    Write-Host "找到启动配置，将执行自启动流程" -ForegroundColor Yellow
    $StartupMode = "bootstrap"
} else {
    Write-Host "未找到启动配置，使用默认配置" -ForegroundColor Yellow
    $StartupMode = "normal"
}

# 启动节点
Write-Host ""
Write-Host "启动节点..." -ForegroundColor Green
Write-Host "启动模式: $StartupMode" -ForegroundColor White
Write-Host ""

# 后台启动
$LogPath = "$LogDir\toki-node.log"
$Process = Start-Process -FilePath "cargo" -ArgumentList "run","--release","--bin","toki-node" -RedirectStandardOutput $LogPath -RedirectStandardError $LogPath -PassThru

Write-Host "节点已启动，PID: $($Process.Id)" -ForegroundColor Green
Write-Host "日志文件: $LogPath" -ForegroundColor White

# 等待节点启动
Write-Host "等待节点启动..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# 检查节点状态
Write-Host "检查节点状态..." -ForegroundColor Green
try {
    $Response = Invoke-RestMethod -Uri "http://localhost:8080/health" -TimeoutSec 5
    Write-Host "✅ 节点启动成功" -ForegroundColor Green
    Write-Host ""
    Write-Host "访问地址:" -ForegroundColor Cyan
    Write-Host "  - 健康检查: http://$ServerIP:8080/health" -ForegroundColor White
    Write-Host "  - 节点信息: http://$ServerIP:8080/api/v1/node/info" -ForegroundColor White
    Write-Host "  - 最新区块: http://$ServerIP:8080/api/v1/block/latest" -ForegroundColor White
    Write-Host ""
    Write-Host "P2P 地址:" -ForegroundColor Cyan
    Write-Host "  - /ip4/$ServerIP/tcp/30333" -ForegroundColor White
} catch {
    Write-Host "❌ 节点启动失败，请检查日志" -ForegroundColor Red
    Write-Host "日志: Get-Content $LogPath -Tail 50" -ForegroundColor Yellow
    exit 1
}

# 保存 PID
$Process.Id | Out-File "$DataDir\toki-node.pid"

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  部署完成" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "管理命令:" -ForegroundColor Yellow
Write-Host "  查看日志: Get-Content $LogPath -Tail 50" -ForegroundColor White
Write-Host "  停止节点: Stop-Process -Id $($Process.Id)" -ForegroundColor White
Write-Host "  重启节点: ./deploy_windows.ps1" -ForegroundColor White
Write-Host ""
