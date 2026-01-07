#!/bin/bash

# AutoZig Examples Verification Script
# 批量编译和运行所有示例项目

set -e  # 遇到错误立即退出

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 编译模式配置（稍后由用户选择）
AUTOZIG_MODE=""
BUILD_MODE="debug"  # debug 或 release
CLEAN_BUILD="no"    # yes 或 no (是否全量编译)

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
    local dir=$(basename "$example_dir")
    
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
    
    # 步骤1: 清理（如果需要全量编译）
    if [ "$CLEAN_BUILD" = "yes" ]; then
        log_info "清理构建产物（全量编译）..."
        if cargo clean 2>&1 | grep -q "error"; then
            log_error "$example_name: 清理失败"
            FAILED=$((FAILED + 1))
            cd - > /dev/null
            return 1
        fi
    else
        log_info "跳过清理（增量编译）"
    fi
    
    # 设置编译模式环境变量
    export AUTOZIG_MODE="$AUTOZIG_MODE"
    
    # 步骤2: 编译
    log_info "编译项目 (${BUILD_MODE} 模式)..."
    local build_cmd="cargo build"
    if [ "$BUILD_MODE" = "release" ]; then
        build_cmd="cargo build --release"
    fi
    
    if ! $build_cmd 2>&1 | tee /tmp/build_${example_name}.log; then
        log_error "$example_name: 编译失败"
        echo "查看详细日志: /tmp/build_${example_name}.log"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    # 双重检查：确保日志中没有 "error:" 关键字
    if grep -qE "error:|error\[|could not compile" /tmp/build_${example_name}.log; then
        log_error "$example_name: 编译过程中检测到错误"
        echo "查看详细日志: /tmp/build_${example_name}.log"
        FAILED=$((FAILED + 1))
        cd - > /dev/null
        return 1
    fi
    log_success "$example_name: 编译成功"
    
    # 步骤3: 运行
    log_info "运行项目 (${BUILD_MODE} 模式)..."
    
    local run_cmd="cargo run"
    if [ "$BUILD_MODE" = "release" ]; then
        run_cmd="cargo run --release"
    fi
    
    local run_success=false
    # 直接尝试cargo run（通常能找到默认binary）
    if timeout 30s $run_cmd 2>&1 | tee /tmp/run_${example_name}.log; then
        run_success=true
    fi
    
    # 检查运行结果
    if [ "$run_success" = true ]; then
        # 额外检查日志中是否有真正的错误（即使exit code为0）
        # 排除误报：
        # - "All tests passed"等成功消息
        # - "Received error"等测试预期错误
        # - Parser/warning等非致命消息
        if grep -E "error:|error\[|could not compile|panicked at" /tmp/run_${example_name}.log | grep -vE "(All .* passed|Received error:|Parser:|warning:)" > /dev/null; then
            log_error "$example_name: 运行过程中检测到错误"
            echo "查看详细日志: /tmp/run_${example_name}.log"
            FAILED=$((FAILED + 1))
            cd - > /dev/null
            return 1
        fi
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
    
    # 检测是否在CI环境中（非交互模式）
    if [ -n "$CI" ] || [ -n "$GITHUB_ACTIONS" ] || [ ! -t 0 ]; then
        # CI模式：使用默认值，不询问用户
        if [ -z "$AUTOZIG_MODE" ]; then
            AUTOZIG_MODE="merged"
        fi
        if [ -z "$BUILD_MODE" ]; then
            BUILD_MODE="debug"
        fi
        if [ -z "$CLEAN_BUILD" ]; then
            CLEAN_BUILD="no"
        fi
        log_info "检测到CI环境，使用默认配置: AUTOZIG_MODE=$AUTOZIG_MODE, BUILD_MODE=$BUILD_MODE, CLEAN_BUILD=$CLEAN_BUILD"
    elif [ -z "$AUTOZIG_MODE" ]; then
        echo -e "${CYAN}================================================${NC}"
        echo -e "${CYAN}  请选择 AutoZig 编译模式${NC}"
        echo -e "${CYAN}================================================${NC}"
        echo ""
        echo -e "${YELLOW}1.${NC} Merged (传统合并模式) ${GREEN}[默认]${NC}"
        echo -e "   └─ 所有Zig代码合并为单文件"
        echo -e "   └─ 向后兼容，适合测试旧代码"
        echo ""
        echo -e "${YELLOW}2.${NC} ModularImport (模块导入模式)"
        echo -e "   └─ 使用@import组织模块"
        echo -e "   └─ 适合纯Zig项目"
        echo ""
        echo -e "${YELLOW}3.${NC} ModularBuildZig (build.zig模式) ${CYAN}⭐ 推荐${NC}"
        echo -e "   └─ 使用Zig构建系统"
        echo -e "   └─ 支持C/Zig混合编程"
        echo -e "   └─ 功能最完整"
        echo ""
        echo -e -n "${BLUE}请输入选项 (1-3) [默认: 1]:${NC} "
        read -r choice
        
        # 处理用户输入
        case "$choice" in
            2)
                AUTOZIG_MODE="modular_import"
                ;;
            3)
                AUTOZIG_MODE="modular_buildzig"
                ;;
            1|"")
                AUTOZIG_MODE="merged"
                ;;
            *)
                log_warning "无效选项 '$choice'，使用默认模式 (Merged)"
                AUTOZIG_MODE="merged"
                ;;
        esac
        echo ""
    fi
    
    # 显示选定的编译模式
    echo -e "${CYAN}================================================${NC}"
    echo -e "${CYAN}  已选择编译模式${NC}"
    echo -e "${CYAN}================================================${NC}"
    case "$AUTOZIG_MODE" in
        merged)
            echo -e "  模式: ${YELLOW}Merged${NC} (传统合并模式)"
            echo -e "  说明: 向后兼容测试，所有Zig代码合并为单文件"
            ;;
        modular_import)
            echo -e "  模式: ${YELLOW}ModularImport${NC} (模块导入模式)"
            echo -e "  说明: 使用@import组织模块，适合纯Zig项目"
            ;;
        modular_buildzig)
            echo -e "  模式: ${YELLOW}ModularBuildZig${NC} (build.zig模式) ${CYAN}⭐${NC}"
            echo -e "  说明: 使用build.zig构建，支持C/Zig混合编程"
            ;;
    esac
    echo -e "${CYAN}================================================${NC}"
    echo -e "${BLUE}提示:${NC} 也可通过环境变量跳过选择："
    echo -e "      ${GREEN}export AUTOZIG_MODE=merged${NC}"
    echo -e "      ${GREEN}export AUTOZIG_MODE=modular_import${NC}"
    echo -e "      ${GREEN}export AUTOZIG_MODE=modular_buildzig${NC}"
    echo ""
    
    # 询问用户选择编译优化模式（仅非CI环境）
    if [ -n "$CI" ] || [ -n "$GITHUB_ACTIONS" ] || [ ! -t 0 ]; then
        # CI环境：使用默认值
        log_info "CI环境：使用默认构建模式 (BUILD_MODE=$BUILD_MODE)"
    else
        echo -e "${CYAN}================================================${NC}"
        echo -e "${CYAN}  选择编译优化模式${NC}"
        echo -e "${CYAN}================================================${NC}"
        echo -e "${YELLOW}1.${NC} Debug (调试模式) ${GREEN}[默认]${NC}"
        echo -e "   └─ 编译速度: ${GREEN}快${NC}"
        echo -e "   └─ 运行速度: 慢"
        echo -e "   └─ 适合: 快速验证、调试错误"
        echo ""
        echo -e "${YELLOW}2.${NC} Release (发布模式)"
        echo -e "   └─ 编译速度: ${RED}慢${NC} (需要优化)"
        echo -e "   └─ 运行速度: ${GREEN}快${NC}"
        echo -e "   └─ 适合: 性能测试、生产环境"
        echo ""
        echo -e -n "${BLUE}请输入选项 (1-2) [默认: 1]:${NC} "
        read -r build_choice
        
        case "$build_choice" in
            2)
                BUILD_MODE="release"
                echo -e "${GREEN}✓ 已选择: Release 模式 (优化编译)${NC}"
                ;;
            1|"")
                BUILD_MODE="debug"
                echo -e "${GREEN}✓ 已选择: Debug 模式 (快速编译)${NC}"
                ;;
            *)
                log_warning "无效选项 '$build_choice'，使用默认 Debug 模式"
                BUILD_MODE="debug"
                ;;
        esac
        echo ""
    fi
    
    # 询问用户选择增量/全量编译（仅非CI环境）
    if [ -n "$CI" ] || [ -n "$GITHUB_ACTIONS" ] || [ ! -t 0 ]; then
        # CI环境：使用默认值
        log_info "CI环境：使用默认编译策略 (CLEAN_BUILD=$CLEAN_BUILD)"
    else
        echo -e "${CYAN}================================================${NC}"
        echo -e "${CYAN}  选择编译方式${NC}"
        echo -e "${CYAN}================================================${NC}"
        echo -e "${YELLOW}1.${NC} 增量编译 (Incremental) ${GREEN}[默认]${NC}"
        echo -e "   └─ 只编译修改的文件"
        echo -e "   └─ 速度: ${GREEN}快${NC} (利用缓存)"
        echo -e "   └─ 适合: 日常开发、快速测试"
        echo ""
        echo -e "${YELLOW}2.${NC} 全量编译 (Clean Build)"
        echo -e "   └─ 清理后重新编译所有文件"
        echo -e "   └─ 速度: ${RED}慢${NC} (从零开始)"
        echo -e "   └─ 适合: 切换编译模式、排查缓存问题"
        echo ""
        echo -e -n "${BLUE}请输入选项 (1-2) [默认: 1]:${NC} "
        read -r clean_choice
        
        case "$clean_choice" in
            2)
                CLEAN_BUILD="yes"
                echo -e "${YELLOW}✓ 已选择: 全量编译 (Clean Build)${NC}"
                ;;
            1|"")
                CLEAN_BUILD="no"
                echo -e "${GREEN}✓ 已选择: 增量编译 (Incremental)${NC}"
                ;;
            *)
                log_warning "无效选项 '$clean_choice'，使用默认增量编译"
                CLEAN_BUILD="no"
                ;;
        esac
        echo ""
    fi
    
    log_info "Examples目录: $SCRIPT_DIR"
    
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
        "Modular Complex (Multi-dir):modular_complex"
        "Stream Support (Phase 4.1):stream_basic"
        "SIMD Detection (Phase 4.2):simd_detect"
        "Zero-Copy Buffer (Phase 4.2):zero_copy"
    )
    
    # WASM 示例（需要 wasm-pack）
    WASM_EXAMPLES=(
        "WASM Image Filter (Phase 5.0):wasm_filter"
    )
    
    # 询问用户选择测试范围（仅非CI环境）
    if [ -n "$CI" ] || [ -n "$GITHUB_ACTIONS" ] || [ ! -t 0 ]; then
        # CI环境：默认测试所有示例
        test_choice="1"
        log_info "CI环境：测试所有示例"
    else
        echo ""
        echo -e "${CYAN}================================================${NC}"
        echo -e "${CYAN}  选择测试范围${NC}"
        echo -e "${CYAN}================================================${NC}"
        echo -e "${YELLOW}1.${NC} 测试所有示例 ${GREEN}[默认]${NC}"
        echo -e "${YELLOW}2.${NC} 选择单个示例测试"
        echo ""
        echo -e -n "${BLUE}请输入选项 (1-2) [默认: 1]:${NC} "
        read -r test_choice
    fi
    
    if [ "$test_choice" = "2" ]; then
        # 显示示例列表供用户选择
        echo ""
        echo -e "${CYAN}可用的示例:${NC}"
        echo ""
        local idx=1
        declare -A example_map
        for example in "${EXAMPLES[@]}"; do
            IFS=':' read -r name dir <<< "$example"
            echo -e "${YELLOW}${idx}.${NC} $name"
            example_map[$idx]="$name:$dir"
            idx=$((idx + 1))
        done
        for example in "${WASM_EXAMPLES[@]}"; do
            IFS=':' read -r name dir <<< "$example"
            echo -e "${YELLOW}${idx}.${NC} $name ${CYAN}(WASM)${NC}"
            example_map[$idx]="$name:$dir:wasm"
            idx=$((idx + 1))
        done
        
        echo ""
        echo -e -n "${BLUE}请选择要测试的示例编号 (1-$((idx-1))):${NC} "
        read -r selected_idx
        
        if [ -n "${example_map[$selected_idx]}" ]; then
            IFS=':' read -r name dir is_wasm <<< "${example_map[$selected_idx]}"
            log_info "开始测试选定示例..."
            if [ "$is_wasm" = "wasm" ]; then
                verify_wasm_example "$name" "$SCRIPT_DIR/$dir"
            else
                verify_example "$name" "$SCRIPT_DIR/$dir"
            fi
        else
            log_error "无效的选择: $selected_idx"
            exit 1
        fi
    else
        # 测试所有示例（默认）
        log_info "开始批量验证..."
        
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
    fi
    
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