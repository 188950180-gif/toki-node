@echo off
REM Toki 主网部署脚本 (Windows)

echo ==========================================
echo   Toki 主网部署
echo ==========================================
echo.

REM 配置
set VERSION=0.1.0
set DATA_DIR=.\mainnet-data
set CONFIG_FILE=mainnet-config.toml

REM 检查依赖
echo 1. 检查依赖...
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo 错误: 未安装 Rust/Cargo
    exit /b 1
)

REM 编译
echo.
echo 2. 编译项目...
cargo build --release
echo 编译完成

REM 创建数据目录
echo.
echo 3. 创建数据目录...
if not exist %DATA_DIR% mkdir %DATA_DIR%
if not exist .\backups mkdir backups
if not exist .\logs mkdir logs

REM 检查配置文件
echo.
echo 4. 检查配置文件...
if not exist %CONFIG_FILE% (
    echo 警告: 未找到 %CONFIG_FILE%
    echo 使用默认配置...
    set CONFIG_FILE=config.toml
)

REM 显示部署信息
echo.
echo ==========================================
echo   部署信息
echo ==========================================
echo 版本:      %VERSION%
echo 数据目录:  %DATA_DIR%
echo 配置文件:  %CONFIG_FILE%
echo 节点程序:  .\target\release\toki-node.exe
echo.

echo 启动命令:
echo   .\target\release\toki-node.exe start --config %CONFIG_FILE%
echo.
