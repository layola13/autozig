#!/bin/bash
# AutoZig v0.1.1 发布脚本
# 自动更新版本号并发布到 crates.io

set -e  # 遇到错误立即退出

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 新版本号
NEW_VERSION="0.1.1"

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}  AutoZig 发布脚本 v${NEW_VERSION}${NC}"
echo -e "${BLUE}======================================${NC}\n"

# 检查是否已登录 crates.io
echo -e "${YELLOW}📋 检查 crates.io 登录状态...${NC}"
if ! grep -q "token" ~/.cargo/credentials.toml 2>/dev/null; then
    echo -e "${RED}❌ 未登录 crates.io！${NC}"
    echo -e "${YELLOW}请先运行: cargo login${NC}"
    exit 1
fi
echo -e "${GREEN}✓ 已登录${NC}\n"

# 函数：更新 Cargo.toml 版本号
update_version() {
    local file=$1
    echo -e "${BLUE}📝 更新版本号: $file${NC}"
    
    # 使用 sed 更新版本号
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \"0\.1\.0\"/version = \"$NEW_VERSION\"/" "$file"
        sed -i '' "s/version = \"0\.1\.0\"/version = \"$NEW_VERSION\"/g" "$file"
    else
        # Linux
        sed -i "s/^version = \"0\.1\.0\"/version = \"$NEW_VERSION\"/" "$file"
        sed -i "s/version = \"0\.1\.0\"/version = \"$NEW_VERSION\"/g" "$file"
    fi
    
    echo -e "${GREEN}✓ 已更新${NC}"
}

# 函数：发布包
publish_package() {
    local package_name=$1
    local package_dir=$2
    
    echo -e "\n${BLUE}======================================${NC}"
    echo -e "${BLUE}  发布: $package_name${NC}"
    echo -e "${BLUE}======================================${NC}\n"
    
    cd "$package_dir"
    
    # Dry run 检查
    echo -e "${YELLOW}🔍 运行 dry-run 检查...${NC}"
    if cargo publish --dry-run; then
        echo -e "${GREEN}✓ Dry-run 通过${NC}"
    else
        echo -e "${RED}❌ Dry-run 失败！${NC}"
        exit 1
    fi
    
    # 实际发布
    echo -e "${YELLOW}📦 发布到 crates.io...${NC}"
    if cargo publish; then
        echo -e "${GREEN}✓ $package_name 发布成功！${NC}"
    else
        echo -e "${RED}❌ $package_name 发布失败！${NC}"
        exit 1
    fi
    
    cd - > /dev/null
}

# 保存当前目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUTOZIG_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$AUTOZIG_ROOT"

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}  步骤 1: 更新版本号${NC}"
echo -e "${BLUE}======================================${NC}\n"

# 更新所有 Cargo.toml 文件
update_version "Cargo.toml"
update_version "parser/Cargo.toml"
update_version "engine/Cargo.toml"
update_version "macro/Cargo.toml"
update_version "gen/build/Cargo.toml"

echo -e "\n${GREEN}✓ 所有版本号已更新为 $NEW_VERSION${NC}"

# 提交版本更新到 Git
echo -e "\n${BLUE}======================================${NC}"
echo -e "${BLUE}  步骤 2: 提交版本更新${NC}"
echo -e "${BLUE}======================================${NC}\n"

cd "$AUTOZIG_ROOT"
echo -e "${YELLOW}📝 提交版本更新到 Git...${NC}"
# 只提交主要的4个 Cargo.toml，gen/build 在 .gitignore 中
git add Cargo.toml parser/Cargo.toml engine/Cargo.toml macro/Cargo.toml
# 强制添加 gen/build/Cargo.toml（即使在 .gitignore 中）
git add -f gen/build/Cargo.toml
# 检查是否有改动需要提交
if git diff --cached --quiet; then
    echo -e "${YELLOW}⚠️  版本号已是最新，无需提交${NC}"
else
    git commit -m "chore: bump version to ${NEW_VERSION}"
    echo -e "${GREEN}✓ 已提交${NC}"
fi

# 运行测试
echo -e "\n${BLUE}======================================${NC}"
echo -e "${BLUE}  步骤 3: 运行测试${NC}"
echo -e "${BLUE}======================================${NC}\n"

echo -e "${YELLOW}🧪 运行 cargo test...${NC}"
if cargo test --lib --bins; then
    echo -e "${GREEN}✓ 测试通过${NC}"
else
    echo -e "${RED}❌ 测试失败！请修复后再发布${NC}"
    exit 1
fi

# 开始发布流程
echo -e "\n${BLUE}======================================${NC}"
echo -e "${BLUE}  步骤 4: 发布包到 crates.io${NC}"
echo -e "${BLUE}======================================${NC}\n"

echo -e "${YELLOW}📋 发布顺序：${NC}"
echo -e "  1️⃣  autozig-parser (无依赖)"
echo -e "  2️⃣  autozig-engine (依赖 parser)"
echo -e "  3️⃣  autozig-macro (依赖 parser)"
echo -e "  4️⃣  autozig-build (依赖 engine)"
echo -e "  5️⃣  autozig (主包，依赖所有子包)\n"

# 1. 发布 autozig-parser
publish_package "autozig-parser" "$AUTOZIG_ROOT/parser"
echo -e "${YELLOW}⏳ 等待 crates.io 索引更新 (30秒)...${NC}"
sleep 30

# 2. 发布 autozig-engine
publish_package "autozig-engine" "$AUTOZIG_ROOT/engine"
echo -e "${YELLOW}⏳ 等待 crates.io 索引更新 (30秒)...${NC}"
sleep 30

# 3. 发布 autozig-macro
publish_package "autozig-macro" "$AUTOZIG_ROOT/macro"
echo -e "${YELLOW}⏳ 等待 crates.io 索引更新 (30秒)...${NC}"
sleep 30

# 4. 发布 autozig-build
publish_package "autozig-build" "$AUTOZIG_ROOT/gen/build"
echo -e "${YELLOW}⏳ 等待 crates.io 索引更新 (30秒)...${NC}"
sleep 30

# 5. 发布主包 autozig
publish_package "autozig" "$AUTOZIG_ROOT"

# 创建 Git 标签
echo -e "\n${BLUE}======================================${NC}"
echo -e "${BLUE}  步骤 5: 创建 Git 标签${NC}"
echo -e "${BLUE}======================================${NC}\n"

cd "$AUTOZIG_ROOT"
echo -e "${YELLOW}🏷️  创建 Git 标签 v${NEW_VERSION}...${NC}"
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION} - WebAssembly support with SIMD optimization"
echo -e "${GREEN}✓ 标签已创建${NC}"

echo -e "\n${YELLOW}推送到 GitHub:${NC}"
echo -e "  git push origin main"
echo -e "  git push origin v${NEW_VERSION}"

# 完成
echo -e "\n${BLUE}======================================${NC}"
echo -e "${BLUE}  发布完成！🎉${NC}"
echo -e "${BLUE}======================================${NC}\n"

echo -e "${GREEN}✓ 所有包已成功发布到 crates.io${NC}"
echo -e "${GREEN}✓ Git 标签 v${NEW_VERSION} 已创建${NC}\n"

echo -e "${YELLOW}后续步骤：${NC}"
echo -e "  1. 推送代码和标签到 GitHub:"
echo -e "     ${BLUE}git push origin main${NC}"
echo -e "     ${BLUE}git push origin v${NEW_VERSION}${NC}"
echo -e "  2. 在 GitHub 上创建 Release"
echo -e "  3. 验证 crates.io 页面:"
echo -e "     ${BLUE}https://crates.io/crates/autozig${NC}"
echo -e "  4. 验证文档:"
echo -e "     ${BLUE}https://docs.rs/autozig${NC}\n"

echo -e "${GREEN}🎊 AutoZig v${NEW_VERSION} 发布成功！${NC}\n"