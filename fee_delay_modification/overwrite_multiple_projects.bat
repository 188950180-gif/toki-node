@echo off
chcp 65001 >nul
REM ============================================================================
REM 多文件夹覆盖脚本 (Windows)
REM ============================================================================

echo ==========================================
echo 多文件夹交易服务费延迟修改
echo ==========================================
echo.

REM 显示项目列表
echo 发现以下项目：
echo 1. tokipt (主项目)
echo 2. tokipt-fixed (修复版)
echo 3. tokipt-backup (备份，不建议修改)
echo.

set /p choices="请选择要覆盖的项目（输入数字，多个用空格分隔，如: 1 2）: "

REM 解析选择
set cover_tokipt=0
set cover_fixed=0
set cover_backup=0

for %%a in (%choices%) do (
    if %%a==1 set cover_tokipt=1
    if %%a==2 set cover_fixed=1
    if %%a==3 set cover_backup=1
)

REM 显示选择
echo.
echo 将覆盖以下项目：
if %cover_tokipt%==1 echo   - ..\tokipt
if %cover_fixed%==1 echo   - ..\tokipt-fixed
if %cover_backup%==1 echo   - ..\tokipt-backup
echo.

set /p confirm="确认继续？(y/n): "
if /i not "%confirm%"=="y" (
    echo 已取消
    pause
    exit /b 0
)

REM 覆盖函数
goto :main

:overwrite_project
setlocal
set project=%~1
set project_name=%~2

echo.
echo ==========================================
echo 覆盖项目: %project_name%
echo ==========================================

REM 检查项目是否存在
if not exist "%project%" (
    echo ❌ 项目不存在: %project%
    exit /b 1
)

REM 检查 fee_delay_modification 目录
if not exist "%project%\fee_delay_modification" (
    echo ❌ fee_delay_modification 目录不存在: %project%
    echo    请先复制 fee_delay_modification 目录到该项目
    exit /b 1
)

REM 覆盖文件
echo 覆盖文件...

copy /Y "%project%\fee_delay_modification\constants.rs" "%project%\core\src\constants.rs" >nul
echo   ✅ core/src/constants.rs

copy /Y "%project%\fee_delay_modification\block.rs" "%project%\core\src\block.rs" >nul
echo   ✅ core/src/block.rs

copy /Y "%project%\fee_delay_modification\transaction.rs" "%project%\core\src\transaction.rs" >nul
echo   ✅ core/src/transaction.rs

copy /Y "%project%\fee_delay_modification\validator.rs" "%project%\consensus\src\validator.rs" >nul
echo   ✅ consensus/src/validator.rs

copy /Y "%project%\fee_delay_modification\block_store.rs" "%project%\storage\src\block_store.rs" >nul
echo   ✅ storage/src/block_store.rs

copy /Y "%project%\fee_delay_modification\handlers.rs" "%project%\api\src\handlers.rs" >nul
echo   ✅ api/src/handlers.rs

copy /Y "%project%\fee_delay_modification\routes.rs" "%project%\api\src\routes.rs" >nul
echo   ✅ api/src/routes.rs

copy /Y "%project%\fee_delay_modification\genesis.rs" "%project%\core\src\genesis.rs" >nul
echo   ✅ core/src/genesis.rs

copy /Y "%project%\fee_delay_modification\tx_pool.rs" "%project%\consensus\src\tx_pool.rs" >nul
echo   ✅ consensus/src/tx_pool.rs

echo ✅ %project_name% 覆盖完成
exit /b 0

:main
REM 执行覆盖
if %cover_tokipt%==1 call :overwrite_project "..\tokipt" "tokipt"
if %cover_fixed%==1 call :overwrite_project "..\tokipt-fixed" "tokipt-fixed"
if %cover_backup%==1 (
    echo.
    set /p confirm_backup="⚠️  确定要覆盖备份吗？(yes/no): "
    if /i "!confirm_backup!"=="yes" (
        call :overwrite_project "..\tokipt-backup" "tokipt-backup"
    ) else (
        echo 跳过备份目录
    )
)

echo.
echo ==========================================
echo ✅ 所有项目覆盖完成！
echo ==========================================
echo.

set /p compile="是否对所有项目进行编译验证？(y/n): "
if /i "%compile%"=="y" (
    if %cover_tokipt%==1 (
        echo.
        echo 编译验证: tokipt
        cd ..\tokipt
        cargo build --workspace --release
        if %errorlevel% equ 0 (
            echo ✅ tokipt 编译通过
        ) else (
            echo ❌ tokipt 编译失败
        )
        cd ..\tokipt
    )

    if %cover_fixed%==1 (
        echo.
        echo 编译验证: tokipt-fixed
        cd ..\tokipt-fixed
        cargo build --workspace --release
        if %errorlevel% equ 0 (
            echo ✅ tokipt-fixed 编译通过
        ) else (
            echo ❌ tokipt-fixed 编译失败
        )
        cd ..\tokipt
    )
)

echo.
echo 🎉 完成！
pause
