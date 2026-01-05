#!/bin/bash

# AutoZig Examples Verification Script
# 批量编译和运行所有示例项目

set -e  # 遇到错误立即退出

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 统计变量
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_section() {
    echo -e "\n${BLUE}======================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}======================================${NC}\n"
}

# 检查示例项目
verify_example() {
    local example_name=$1
    local example_dir="$2"
    
    TOTAL=$((TOTAL + 1))
    
    log_section "验证示例: $example_name"
    
    # 检查目录是否存在
    if [ ! -d "$example_dir" ]; then
        log_error "目录不存在: $example_dir"
        SKIPPED=$((SKIPPED + 1))
        return 1
    fi
    
    cd "$example_dir"
    
    # 步骤1: 清理
    log_info "清理构建产物..."
    if cargo clean 2>&1 | grep -q "error"; then
        log_error "$example_name: 清理失败"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    
    # 步骤2: 编译
    log_info "编译项目..."
    if cargo build 2>&1 | tee /tmp/build_${example_name}.log | grep -q "error\["; then
        log_error "$example_name: 编译失败"
        echo "查看详细日志: /tmp/build_${example_name}.log"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    log_success "$example_name: 编译成功"
    
    # 步骤3: 运行
    log_info "运行项目..."
    if timeout 30s cargo run 2>&1 | tee /tmp/run_${example_name}.log; then
        log_success "$example_name: 运行成功"
        PASSED=$((PASSED + 1))
    else
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 124 ]; then
            log_error "$example_name: 运行超时（30秒）"
        else
            log_error "$example_name: 运行失败 (退出码: $EXIT_CODE)"
        fi
        echo "查看详细日志: /tmp/run_${example_name}.log"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    
    cd - > /dev/null
    return 0
}

# 主函数
main() {
    log_section "AutoZig Examples 验证工具"
    
    # 获取脚本所在目录（examples目录）
    SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    
    log_info "Examples目录: $SCRIPT_DIR"
    log_info "开始批量验证..."
    
    # 定义所有示例项目
    # 格式: "显示名称:目录名"
    EXAMPLES=(
        "Structs Example:structs"
        "Enums Example:enums"
        "Complex Types:complex"
        "Smart Lowering:smart_lowering"
        "External Zig:external"
        "Trait Calculator:trait_calculator"
        "Trait Hasher:trait_hasher"
        "Security Tests:security_tests"
        "Generics (Phase 3):generics"
        "Async FFI (Phase 3):async"
        "Zig-C Interop:zig-c"
    )
    
    # 遍历所有示例
    for example in "${EXAMPLES[@]}"; do
        IFS=':' read -r name dir <<< "$example"
        verify_example "$name" "$SCRIPT_DIR/$dir"
    done
    
    # 输出总结
    log_section "验证结果总结"
    echo "总计: $TOTAL 个示例"
    echo -e "${GREEN}成功: $PASSED${NC}"
    echo -e "${RED}失败: $FAILED${NC}"
    echo -e "${YELLOW}跳过: $SKIPPED${NC}"
    
    if [ $FAILED -eq 0 ]; then
        log_success "所有示例验证通过！🎉"
        exit 0
    else
        log_error "有 $FAILED 个示例验证失败"
        exit 1
    fi
}

# 运行主函数
main "$@"