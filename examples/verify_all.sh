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

# 检查 main.rs 或 lib.rs 是否包含 AutoZig 宏
check_autozig_macro() {
    local example_dir=$1
    local main_rs="$example_dir/src/main.rs"
    local lib_rs="$example_dir/src/lib.rs"
    
    # 优先检查 main.rs，如果不存在则检查 lib.rs（WASM项目）
    if [ -f "$main_rs" ]; then
        local source_file="$main_rs"
    elif [ -f "$lib_rs" ]; then
        local source_file="$lib_rs"
        log_info "检测到 lib.rs (WASM 项目)"
    else
        log_error "找不到 main.rs 或 lib.rs 文件"
        return 1
    fi
    
    # 检查是否包含 autozig! 或 include_zig! 宏
    if grep -qE '(autozig!|include_zig!)' "$source_file"; then
        log_success "检测到 AutoZig 宏 (autozig! 或 include_zig!)"
        return 0
    else
        log_error "源文件缺少必需的 AutoZig 宏 (autozig! 或 include_zig!)"
        return 1
    fi
}

# 检查 WASM 示例项目（特殊处理）
verify_wasm_example() {
    local example_name=$1
    local example_dir="$2"
    
    TOTAL=$((TOTAL + 1))
    
    log_section "验证 WASM 示例: $example_name"
    
    # 检查目录是否存在
    if [ ! -d "$example_dir" ]; then
        log_error "目录不存在: $example_dir"
        SKIPPED=$((SKIPPED + 1))
        return 1
    fi
    
    # 检查 wasm-pack 是否安装
    if ! command -v wasm-pack &> /dev/null; then
        log_warning "wasm-pack 未安装，跳过 WASM 示例"
        log_info "安装方法: cargo install wasm-pack"
        SKIPPED=$((SKIPPED + 1))
        return 1
    fi
    
    cd "$example_dir"
    
    # 步骤0: 检查 AutoZig 宏
    log_info "检查 AutoZig 宏使用..."
    if ! check_autozig_macro "$example_dir"; then
        log_error "$example_name: 宏检查失败"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    
    # 步骤1: 清理
    log_info "清理构建产物..."
    if cargo clean 2>&1 | grep -q "error"; then
        log_error "$example_name: 清理失败"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    
    # 步骤2: 使用 wasm-pack 编译
    log_info "使用 wasm-pack 编译 WASM..."
    if wasm-pack build --target web --release 2>&1 | tee /tmp/build_${example_name}.log | grep -qE "(error\[|Error)"; then
        log_error "$example_name: WASM 编译失败"
        echo "查看详细日志: /tmp/build_${example_name}.log"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    log_success "$example_name: WASM 编译成功"
    
    # 步骤3: 检查生成的 WASM 文件
    if [ -f "pkg/*.wasm" ] || [ -d "pkg" ]; then
        log_success "$example_name: WASM 包生成成功 (pkg/)"
        PASSED=$((PASSED + 1))
    else
        log_error "$example_name: 未找到生成的 WASM 包"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    
    cd - > /dev/null
    return 0
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
    
    # 步骤0: 检查 AutoZig 宏
    log_info "检查 AutoZig 宏使用..."
    if ! check_autozig_macro "$example_dir"; then
        log_error "$example_name: 宏检查失败"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    
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
        "Stream Support (Phase 4.1):stream_basic"
        "SIMD Detection (Phase 4.2):simd_detect"
        "Zero-Copy Buffer (Phase 4.2):zero_copy"
    )
    
    # WASM 示例（需要 wasm-pack）
    WASM_EXAMPLES=(
        "WASM Image Filter (Phase 5.0):wasm_filter"
    )
    
    # 遍历所有标准示例
    for example in "${EXAMPLES[@]}"; do
        IFS=':' read -r name dir <<< "$example"
        verify_example "$name" "$SCRIPT_DIR/$dir"
    done
    
    # 遍历 WASM 示例
    for example in "${WASM_EXAMPLES[@]}"; do
        IFS=':' read -r name dir <<< "$example"
        verify_wasm_example "$name" "$SCRIPT_DIR/$dir"
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