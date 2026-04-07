@echo off
chcp 65001 >nul

echo ==========================================
echo Toki 节点远程部署
echo ==========================================
echo.

echo 服务器信息:
echo   IP: 182.254.176.30
echo   主机名: lhins-i50xxnc9
echo   项目路径: /home/uploader/tokipt/
echo.

echo 请按以下步骤操作:
echo.
echo 1. 连接到服务器:
echo    ssh uploader@182.254.176.30
echo.
echo 2. 进入项目目录:
echo    cd /home/uploader/tokipt
echo.
echo 3. 添加执行权限:
echo    chmod +x deploy_and_run.sh quick_start.sh
echo.
echo 4. 运行部署脚本:
echo    ./deploy_and_run.sh
echo.
echo 或者快速启动（如果已编译）:
echo    ./quick_start.sh
echo.
echo 5. 查看运行状态:
echo    tail -f logs/toki.log
echo.
echo 6. 测试API:
echo    curl http://localhost:8080/health
echo    curl http://localhost:8080/api/v1/fee/announcement
echo.

echo ==========================================
echo 部署脚本已准备好
echo ==========================================
echo.

pause
