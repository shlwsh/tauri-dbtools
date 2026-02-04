#!/usr/bin/env bun
/**
 * MyGit - 智能 Git 自动化工作流
 * 
 * 使用 AI 生成提交信息并自动推送到远程仓库
 * 
 * 用法:
 *   bun run mygit                           # AI 生成提交信息
 *   bun run mygit "自定义提交信息"          # 使用自定义信息
 *   bun run mygit --dry-run                 # 预览模式
 *   bun run mygit --branch develop          # 指定分支
 */

import { $ } from 'bun';
import { parseArgs } from 'util';

// 颜色定义
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
};

// 输出函数
const print = {
  success: (msg: string) => console.log(`${colors.green}✓ ${msg}${colors.reset}`),
  info: (msg: string) => console.log(`${colors.cyan}ℹ ${msg}${colors.reset}`),
  warning: (msg: string) => console.log(`${colors.yellow}⚠ ${msg}${colors.reset}`),
  error: (msg: string) => console.log(`${colors.red}✗ ${msg}${colors.reset}`),
};

// 配置接口
interface Config {
  message?: string;
  branch?: string;
  remote: string;
  dryRun: boolean;
}

// 文件状态接口
interface FileStatus {
  status: string;
  file: string;
}

/**
 * 解析命令行参数
 */
function parseArguments(): Config {
  const args = process.argv.slice(2);
  
  const config: Config = {
    remote: '', // 将在后面自动检测
    dryRun: false,
  };

  // 解析参数
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    if (arg === '--dry-run') {
      config.dryRun = true;
    } else if (arg === '--branch' && i + 1 < args.length) {
      config.branch = args[++i];
    } else if (arg === '--remote' && i + 1 < args.length) {
      config.remote = args[++i];
    } else if (!arg.startsWith('--') && !config.message) {
      config.message = arg;
    }
  }

  return config;
}

/**
 * 检查是否在 Git 仓库中
 */
async function checkGitRepository(): Promise<boolean> {
  try {
    await $`git rev-parse --git-dir`.quiet();
    return true;
  } catch {
    return false;
  }
}

/**
 * 获取当前分支
 */
async function getCurrentBranch(): Promise<string> {
  try {
    const result = await $`git rev-parse --abbrev-ref HEAD`.text();
    return result.trim();
  } catch {
    throw new Error('无法获取当前分支');
  }
}

/**
 * 获取默认远程仓库
 */
async function getDefaultRemote(): Promise<string> {
  try {
    // 尝试获取所有远程仓库
    const result = await $`git remote`.text();
    const remotes = result.trim().split('\n').filter(Boolean);
    
    if (remotes.length === 0) {
      throw new Error('没有配置远程仓库');
    }
    
    // 优先使用 origin，否则使用第一个
    if (remotes.includes('origin')) {
      return 'origin';
    }
    
    return remotes[0];
  } catch {
    throw new Error('无法获取远程仓库');
  }
}

/**
 * 获取 Git 状态
 */
async function getGitStatus(): Promise<string> {
  try {
    const result = await $`git status --porcelain`.text();
    return result;
  } catch {
    return '';
  }
}

/**
 * 获取修改的文件列表
 */
async function getModifiedFiles(): Promise<string[]> {
  try {
    const files = new Set<string>();

    // 获取已修改的文件
    const modified = await $`git diff --name-only HEAD`.text();
    if (modified) {
      modified.split('\n').filter(Boolean).forEach(f => files.add(f));
    }

    // 获取已暂存的文件
    const staged = await $`git diff --cached --name-only`.text();
    if (staged) {
      staged.split('\n').filter(Boolean).forEach(f => files.add(f));
    }

    // 获取未跟踪的文件
    const untracked = await $`git ls-files --others --exclude-standard`.text();
    if (untracked) {
      untracked.split('\n').filter(Boolean).forEach(f => files.add(f));
    }

    return Array.from(files);
  } catch {
    return [];
  }
}

/**
 * 获取文件状态
 */
async function getFileStatus(file: string): Promise<string> {
  try {
    const result = await $`git status --porcelain ${file}`.text();
    if (!result) return '变更';
    
    const statusCode = result.substring(0, 2).trim();
    const statusMap: Record<string, string> = {
      'M': '修改',
      'A': '新增',
      'D': '删除',
      'R': '重命名',
      '??': '未跟踪',
    };
    
    return statusMap[statusCode] || '变更';
  } catch {
    return '变更';
  }
}

/**
 * 获取变更摘要
 */
async function getDiffSummary(files: string[]): Promise<string> {
  const summary: string[] = [];
  
  for (const file of files) {
    const status = await getFileStatus(file);
    summary.push(`- [${status}] ${file}`);
  }
  
  return summary.join('\n');
}

/**
 * 分析文件类型
 */
function analyzeFiles(files: string[]): {
  hasTests: boolean;
  hasDocs: boolean;
  hasFrontend: boolean;
  hasBackend: boolean;
  hasConfig: boolean;
} {
  return {
    hasTests: files.some(f => /test|spec/.test(f)),
    hasDocs: files.some(f => /\.md$|README/.test(f)),
    hasFrontend: files.some(f => /frontend\//.test(f)),
    hasBackend: files.some(f => /src-tauri\//.test(f)),
    hasConfig: files.some(f => /\.(json|toml|yaml|yml)$/.test(f)),
  };
}

/**
 * 生成 AI 提交信息
 */
async function generateCommitMessage(files: string[]): Promise<string> {
  print.info('正在分析变更并生成提交信息...');
  
  const analysis = analyzeFiles(files);
  const fileCount = files.length;
  
  // 生成提交信息
  let type = 'chore';
  let scope = '';
  let description = '更新项目文件';
  
  if (analysis.hasTests) {
    type = 'test';
    scope = 'tests';
    description = '完善测试覆盖率';
  } else if (analysis.hasDocs) {
    type = 'docs';
    description = '更新文档';
  } else if (analysis.hasFrontend && analysis.hasBackend) {
    type = 'feat';
    scope = 'fullstack';
    description = '更新前后端功能';
  } else if (analysis.hasFrontend) {
    type = 'feat';
    scope = 'frontend';
    description = '更新前端功能';
  } else if (analysis.hasBackend) {
    type = 'feat';
    scope = 'backend';
    description = '更新后端功能';
  } else if (analysis.hasConfig) {
    type = 'chore';
    scope = 'config';
    description = '更新配置文件';
  }
  
  // 构建提交信息
  const commitMessage = scope 
    ? `${type}(${scope}): ${description}`
    : `${type}: ${description}`;
  
  return `${commitMessage}\n\n变更文件数: ${fileCount}`;
}

/**
 * 询问用户确认
 */
async function askConfirmation(message: string): Promise<boolean> {
  const prompt = `${message} (Y/n): `;
  process.stdout.write(prompt);
  
  // 读取用户输入
  for await (const line of console) {
    const input = line.toString().trim().toLowerCase();
    if (input === '' || input === 'y' || input === 'yes') {
      return true;
    }
    if (input === 'n' || input === 'no') {
      return false;
    }
    process.stdout.write(prompt);
  }
  
  return true;
}

/**
 * 获取用户输入
 */
async function getUserInput(prompt: string): Promise<string> {
  process.stdout.write(`${prompt}: `);
  
  for await (const line of console) {
    return line.toString().trim();
  }
  
  return '';
}

/**
 * 执行 Git 命令
 */
async function executeGitCommand(command: string, description: string): Promise<boolean> {
  try {
    print.info(description);
    await $`${command}`.quiet();
    print.success('完成');
    return true;
  } catch (error) {
    print.error(`失败: ${error}`);
    return false;
  }
}

/**
 * 主函数
 */
async function main() {
  print.info('=== Git 自动化工作流 ===');
  console.log();
  
  // 解析参数
  const config = parseArguments();
  
  // 检查 Git 仓库
  if (!(await checkGitRepository())) {
    print.error('当前目录不是 Git 仓库');
    process.exit(1);
  }
  print.success('Git 仓库检查通过');
  
  // 获取当前分支
  const currentBranch = await getCurrentBranch();
  const targetBranch = config.branch || currentBranch;
  
  // 获取远程仓库（如果未指定）
  if (!config.remote) {
    try {
      config.remote = await getDefaultRemote();
    } catch (error) {
      print.error(`${error}`);
      print.info('提示: 使用 --remote 参数指定远程仓库');
      process.exit(1);
    }
  }
  
  print.info(`当前分支: ${currentBranch}`);
  print.info(`目标分支: ${targetBranch}`);
  print.info(`远程仓库: ${config.remote}`);
  console.log();
  
  // 检查是否有变更
  const status = await getGitStatus();
  if (!status) {
    print.warning('没有检测到任何变更');
    print.info('工作区是干净的，无需提交');
    process.exit(0);
  }
  
  print.success('检测到文件变更');
  
  // 获取修改的文件
  const modifiedFiles = await getModifiedFiles();
  if (modifiedFiles.length === 0) {
    print.warning('没有找到修改的文件');
    process.exit(0);
  }
  
  print.info(`变更文件数: ${modifiedFiles.length}`);
  console.log();
  
  // 显示变更摘要
  print.info('变更摘要:');
  const diffSummary = await getDiffSummary(modifiedFiles);
  console.log(diffSummary);
  console.log();
  
  // 生成或使用提交信息
  let commitMessage = config.message;
  
  if (!commitMessage) {
    commitMessage = await generateCommitMessage(modifiedFiles);
    print.success('AI 生成的提交信息:');
    console.log(`${colors.yellow}${commitMessage}${colors.reset}`);
    console.log();
    
    if (!config.dryRun) {
      // 简化确认流程 - 直接使用生成的信息
      const useGenerated = await askConfirmation('是否使用此提交信息？');
      if (!useGenerated) {
        commitMessage = await getUserInput('请输入自定义提交信息');
      }
    }
  } else {
    print.info(`使用自定义提交信息: ${commitMessage}`);
  }
  
  console.log();
  
  // 预览模式
  if (config.dryRun) {
    print.warning('=== 预览模式 - 不会执行实际操作 ===');
    print.info('将执行以下操作:');
    console.log('  1. git add .');
    console.log(`  2. git commit -m "${commitMessage}"`);
    console.log(`  3. git push ${config.remote} ${targetBranch}`);
    process.exit(0);
  }
  
  // 执行 Git 操作
  print.info('=== 开始执行 Git 操作 ===');
  console.log();
  
  // 1. 添加所有更改
  print.info('步骤 1/3: 添加所有更改...');
  try {
    await $`git add .`.quiet();
    print.success('已添加所有更改');
  } catch (error) {
    print.error('添加文件失败');
    process.exit(1);
  }
  
  // 2. 提交更改
  print.info('步骤 2/3: 提交更改...');
  try {
    await $`git commit -m ${commitMessage}`.quiet();
    print.success('提交成功');
  } catch (error) {
    print.error('提交失败');
    process.exit(1);
  }
  
  // 3. 推送到远程仓库
  print.info('步骤 3/3: 推送到远程仓库...');
  try {
    await $`git push ${config.remote} ${targetBranch}`.quiet();
    print.success('推送成功');
    console.log();
    print.success('=== 所有操作完成 ===');
    print.info(`提交信息: ${commitMessage.split('\n')[0]}`);
    print.info(`分支: ${targetBranch}`);
    print.info(`远程: ${config.remote}`);
  } catch (error) {
    print.error('推送失败');
    console.error(error);
    process.exit(1);
  }
}

// 运行主函数
main().catch((error) => {
  print.error(`发生未预期的错误: ${error}`);
  process.exit(1);
});
