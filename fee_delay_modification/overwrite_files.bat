@echo off
chcp 65001 >nul
REM ============================================================================
REM 交易服务费延迟修改方案 - 一键覆盖脚本 (Windows)
REM ============================================================================

echo ==========================================
echo 交易服务费延迟修改方案 - 文件覆盖
echo ==========================================
echo.

REM 检查是否在项目根目录
if not exist "core" (
    echo ❌ 错误：请在项目根目录执行此脚本
    exit /b 1
)

REM 检查修改文件是否存在
if not exist "fee_delay_modification" (
    echo ❌ 错误：fee_delay_modification 目录不存在
    exit /b 1
)

echo 开始覆盖文件...
echo.

REM 1. 核心常量
echo 1️⃣  覆盖 constants.rs...
copy /Y fee_delay_modification\constants.rs core\src\constants.rs >nul
echo    ✅ core/src/constants.rs 已覆盖

REM 2. 区块结构
echo 2️⃣  覆盖 block.rs...
copy /Y fee_delay_modification\block.rs core\src\block.rs >nul
echo    ✅ core/src/block.rs 已覆盖

REM 3. 交易逻辑
echo 3️⃣  覆盖 transaction.rs...
copy /Y fee_delay_modification\transaction.rs core\src\transaction.rs >nul
echo    ✅ core/src/transaction.rs 已覆盖

REM 4. 交易验证
echo 4️⃣  覆盖 validator.rs...
copy /Y fee_delay_modification\validator.rs consensus\src\validator.rs >nul
echo    ✅ consensus/src/validator.rs 已覆盖

REM 5. 区块存储
echo 5️⃣  覆盖 block_store.rs...
copy /Y fee_delay_modification\block_store.rs storage\src\block_store.rs >nul
echo    ✅ storage/src/block_store.rs 已覆盖

REM 6. API处理器
echo 6️⃣  覆盖 handlers.rs...
copy /Y fee_delay_modification\handlers.rs api\src\handlers.rs >nul
echo    ✅ api/src/handlers.rs 已覆盖

REM 7. API路由
echo 7️⃣  覆盖 routes.rs...
copy /Y fee_delay_modification\routes.rs api\src\routes.rs >nul
echo    ✅ api/src/routes.rs 已覆盖

REM 8. 创世配置
echo 8️⃣  覆盖 genesis.rs...
copy /Y fee_delay_modification\genesis.rs core\src\genesis.rs >nul
echo    ✅ core/src/genesis.rs 已覆盖

REM 9. 交易池配置
echo 9️⃣  覆盖 tx_pool.rs...
copy /Y fee_delay_modification\tx_pool.rs consensus\src\tx_pool.rs >nul
echo    ✅ consensus/src/tx_pool.rs 已覆盖

echo.
echo ==========================================
echo ✅ 所有文件已覆盖完成！
echo ==========================================
echo.

REM 询问是否编译验证
set /p compile="是否进行编译验证？(y/n): "
if /i "%compile%"=="y" (
    echo 开始编译验证...
    echo.
    cargo build --workspace --release
    if %errorlevel% equ 0 (
        echo.
        echo ==========================================
        echo ✅ 编译验证通过！
        echo ==========================================
    ) else (
        echo.
        echo ==========================================
        echo ❌ 编译验证失败！
        echo ==========================================
        exit /b 1
    )
)

echo.
echo 🎉 文件覆盖完成！
echo.
echo 下一步：
echo 1. 运行测试: cargo test --workspace
echo 2. 验证功能: 检查交易费用延迟逻辑
echo 3. 提交代码: git add . ^&^& git commit -m "feat: 添加交易服务费延迟180天功能"
echo.

pause
