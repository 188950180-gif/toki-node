@echo off
REM Toki 测试网启动脚本 (Windows)

echo === Toki 测试网启动 ===
echo.

REM 检查可执行文件
if not exist ".\target\release\toki-node.exe" (
    echo 错误: 未找到 toki-node 可执行文件
    echo 请先运行: cargo build --release
    exit /b 1
)

REM 创建数据目录
if not exist ".\testnet-data" mkdir testnet-data

REM 检查创世区块
if not exist ".\testnet-genesis.json" (
    echo 错误: 未找到创世区块文件
    exit /b 1
)

echo 配置文件: testnet-config.toml
echo 创世区块: testnet-genesis.json
echo 数据目录: .\testnet-data
echo.

REM 启动节点
echo 启动测试网节点...
.\target\release\toki-node.exe start --config testnet-config.toml
