#!/bin/bash
#
# 自动化 Git 工作流 - 使用 AI 生成提交信息并推送到远程仓库
#
# 用法:
#   ./mygit.sh                    # 使用 AI 生成提交信息
#   ./mygit.sh "自定义提交信息"    # 使用自定义提交信息
#   ./mygit.sh --dry-run          # 预览模式

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 输出函数
print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_info() { echo -e "${CYAN}ℹ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠ $1${NC}"; }
print_error() { echo -e "${RED}✗ $1${NC}"; }

# 参数解析
MESSAGE=""
DRY_RUN=false
REMOTE="origin"
BRANCH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --remote)
            REMOTE="$2"
            shift 2
            ;;
        --branch)
            BRANCH="$2"
            shift 2
            ;;
        *)
            MESSAGE="$1"
            shift
            ;;
    esac
done

# 检查是否在 Git 仓库中
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_error "当前目录不是 Git 仓库"
        exit 1
    fi
}

# 获取当前分支
get_current_branch() {
    git rev-parse --abbrev-ref HEAD
}

# 获取修改的文件
get_modified_files() {
    git diff --name-only HEAD
    git diff --cached --name-only
    git ls-files --others --exclude-standard
}

# 获取变更摘要
get_diff_summary() {
    local files=("$@")
    local summary=""
    
    for file in "${files[@]}"; do
        local status=$(git status --porcelain "$file" 2>/dev/null | cut -c1-2)
        local status_text=""
        
        case "$status" in
            "M"|" M") status_text="修改" ;;
            "A"|"??") status_text="新增" ;;
            "D"|" D") status_text="删除" ;;
            "R") status_text="重命名" ;;
            *) status_text="变更" ;;
        esac
        
        summary="${summary}- [${status_text}] ${file}\n"
    done
    
    echo -e "$summary"
}

# 生成 AI 提交信息
generate_commit_message() {
    local files=("$@")
    local file_count=${#files[@]}
    
    print_info "正在分析变更并生成提交信息..."
    
    # 分析文件类型
    local has_tests=false
    local has_docs=false
    local has_frontend=false
    local has_backend=false
    local has_config=false
    
    for file in "${files[@]}"; do
        [[ "$file" =~ test|spec ]] && has_tests=true
        [[ "$file" =~ \.md$|README ]] && has_docs=true
        [[ "$file" =~ frontend/ ]] && has_frontend=true
        [[ "$file" =~ src-tauri/ ]] && has_backend=true
        [[ "$file" =~ \.(json|toml|yaml)$ ]] && has_config=true
    done
    
    # 生成提交信息
    local type="chore"
    local scope=""
    local description="更新项目文件"
    
    if $has_tests; then
        type="test"
        scope="tests"
        description="完善测试覆盖率"
    elif $has_docs; then
        type="docs"
        description="更新文档"
    elif $has_frontend && $has_backend; then
        type="feat"
        scope="fullstack"
        description="更新前后端功能"
    elif $has_frontend; then
        type="feat"
        scope="frontend"
        description="更新前端功能"
    elif $has_backend; then
        type="feat"
        scope="backend"
        description="更新后端功能"
    elif $has_config; then
        type="chore"
        scope="config"
        description="更新配置文件"
    fi
    
    # 构建提交信息
    if [ -n "$scope" ]; then
        echo "${type}(${scope}): ${description}"
    else
        echo "${type}: ${description}"
    fi
    
    echo ""
    echo "变更文件数: $file_count"
}

# 主函数
main() {
    print_info "=== Git 自动化工作流 ==="
    echo ""
    
    # 检查 Git 仓库
    check_git_repo
    print_success "Git 仓库检查通过"
    
    # 获取当前分支
    local current_branch=$(get_current_branch)
    if [ -z "$BRANCH" ]; then
        BRANCH="$current_branch"
    fi
    
    print_info "当前分支: $current_branch"
    print_info "目标分支: $BRANCH"
    print_info "远程仓库: $REMOTE"
    echo ""
    
    # 检查是否有变更
    if [ -z "$(git status --porcelain)" ]; then
        print_warning "没有检测到任何变更"
        print_info "工作区是干净的，无需提交"
        exit 0
    fi
    
    print_success "检测到文件变更"
    
    # 获取修改的文件
    local modified_files=($(get_modified_files | sort -u))
    if [ ${#modified_files[@]} -eq 0 ]; then
        print_warning "没有找到修改的文件"
        exit 0
    fi
    
    print_info "变更文件数: ${#modified_files[@]}"
    echo ""
    
    # 显示变更摘要
    print_info "变更摘要:"
    get_diff_summary "${modified_files[@]}"
    echo ""
    
    # 生成或使用提交信息
    if [ -z "$MESSAGE" ]; then
        MESSAGE=$(generate_commit_message "${modified_files[@]}")
        print_success "AI 生成的提交信息:"
        echo -e "${YELLOW}${MESSAGE}${NC}"
        echo ""
        
        if ! $DRY_RUN; then
            read -p "是否使用此提交信息？(Y/n): " confirm
            if [[ ! "$confirm" =~ ^[Yy]?$ ]]; then
                read -p "请输入自定义提交信息: " MESSAGE
            fi
        fi
    else
        print_info "使用自定义提交信息: $MESSAGE"
    fi
    
    echo ""
    
    # 预览模式
    if $DRY_RUN; then
        print_warning "=== 预览模式 - 不会执行实际操作 ==="
        print_info "将执行以下操作:"
        echo "  1. git add ."
        echo "  2. git commit -m \"$MESSAGE\""
        echo "  3. git push $REMOTE $BRANCH"
        exit 0
    fi
    
    # 执行 Git 操作
    print_info "=== 开始执行 Git 操作 ==="
    echo ""
    
    # 1. 添加所有更改
    print_info "步骤 1/3: 添加所有更改..."
    if git add .; then
        print_success "已添加所有更改"
    else
        print_error "添加文件失败"
        exit 1
    fi
    
    # 2. 提交更改
    print_info "步骤 2/3: 提交更改..."
    if git commit -m "$MESSAGE"; then
        print_success "提交成功"
    else
        print_error "提交失败"
        exit 1
    fi
    
    # 3. 推送到远程仓库
    print_info "步骤 3/3: 推送到远程仓库..."
    if git push "$REMOTE" "$BRANCH"; then
        print_success "推送成功"
        echo ""
        print_success "=== 所有操作完成 ==="
        print_info "提交信息: $MESSAGE"
        print_info "分支: $BRANCH"
        print_info "远程: $REMOTE"
    else
        print_error "推送失败"
        exit 1
    fi
}

# 运行主函数
main
