#!/bin/bash

# ============================================================================
# 多文件夹覆盖脚本
# ============================================================================

echo "=========================================="
echo "多文件夹交易服务费延迟修改"
echo "=========================================="
echo ""

# 定义项目目录
PROJECTS=(
    "../tokipt"
    "../tokipt-fixed"
)

# 定义修改文件
FILES=(
    "constants.rs:core/src/constants.rs"
    "block.rs:core/src/block.rs"
    "transaction.rs:core/src/transaction.rs"
    "validator.rs:consensus/src/validator.rs"
    "block_store.rs:storage/src/block_store.rs"
    "handlers.rs:api/src/handlers.rs"
    "routes.rs:api/src/routes.rs"
    "genesis.rs:core/src/genesis.rs"
    "tx_pool.rs:consensus/src/tx_pool.rs"
)

# 询问要覆盖哪些项目
echo "发现以下项目："
echo "1. tokipt (主项目)"
echo "2. tokipt-fixed (修复版)"
echo "3. tokipt-backup (备份，不建议修改)"
echo ""
read -p "请选择要覆盖的项目（输入数字，多个用空格分隔，如: 1 2）: " -a choices

# 处理选择
selected_projects=()
for choice in "${choices[@]}"; do
    case $choice in
        1) selected_projects+=("../tokipt") ;;
        2) selected_projects+=("../tokipt-fixed") ;;
        3)
            read -p "⚠️  确定要覆盖备份吗？(yes/no): " confirm
            if [ "$confirm" = "yes" ]; then
                selected_projects+=("../tokipt-backup")
            else
                echo "跳过备份目录"
            fi
            ;;
        *) echo "无效选择: $choice" ;;
    esac
done

if [ ${#selected_projects[@]} -eq 0 ]; then
    echo "❌ 未选择任何项目"
    exit 1
fi

echo ""
echo "将覆盖以下项目："
for proj in "${selected_projects[@]}"; do
    echo "  - $proj"
done
echo ""

read -p "确认继续？(y/n): " confirm
if [ "$confirm" != "y" ]; then
    echo "已取消"
    exit 0
fi

# 开始覆盖
for project in "${selected_projects[@]}"; do
    echo ""
    echo "=========================================="
    echo "覆盖项目: $project"
    echo "=========================================="

    # 检查项目是否存在
    if [ ! -d "$project" ]; then
        echo "❌ 项目不存在: $project"
        continue
    fi

    # 检查 fee_delay_modification 目录
    if [ ! -d "$project/fee_delay_modification" ]; then
        echo "❌ fee_delay_modification 目录不存在: $project"
        echo "   请先复制 fee_delay_modification 目录到该项目"
        continue
    fi

    # 覆盖文件
    for file_mapping in "${FILES[@]}"; do
        IFS=':' read -r src_file dest_path <<< "$file_mapping"

        src="$project/fee_delay_modification/$src_file"
        dest="$project/$dest_path"

        if [ -f "$src" ]; then
            cp "$src" "$dest"
            echo "  ✅ $dest_path"
        else
            echo "  ❌ 源文件不存在: $src_file"
        fi
    done

    echo "✅ $project 覆盖完成"
done

echo ""
echo "=========================================="
echo "✅ 所有项目覆盖完成！"
echo "=========================================="
echo ""

# 询问是否编译验证
read -p "是否对所有项目进行编译验证？(y/n): " compile
if [ "$compile" = "y" ]; then
    for project in "${selected_projects[@]}"; do
        echo ""
        echo "编译验证: $project"
        cd "$project"
        cargo build --workspace --release
        if [ $? -eq 0 ]; then
            echo "✅ $project 编译通过"
        else
            echo "❌ $project 编译失败"
        fi
        cd - > /dev/null
    done
fi

echo ""
echo "🎉 完成！"
