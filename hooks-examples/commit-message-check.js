#!/usr/bin/env node
// PreCommit hook - 检查提交信息格式
// 规则：提交信息必须以 type: 开头
// 用法：在 hooks.toml 中配置 script_path 指向此文件

const context = JSON.parse(process.argv[2]);
const { data } = context;
const message = (data.message || "").trim();

const VALID_TYPES = ["feat", "fix", "docs", "style", "refactor", "test", "chore", "perf", "ci", "build", "revert"];
const PATTERN = new RegExp(`^(${VALID_TYPES.join("|")})(\\(.+\\))?: .+`);

if (!message) {
  console.error("[PreCommit] 提交信息不能为空");
  process.stdout.write('"Cancel"');
  process.exit(0);
}

if (!PATTERN.test(message)) {
  console.error(`[PreCommit] 提交信息格式错误: "${message}"`);
  console.error(`  要求: type: description 或 type(scope): description`);
  console.error(`  允许的 type: ${VALID_TYPES.join(", ")}`);
  console.error(`  示例: fix(login): 修复登录超时问题`);
  process.stdout.write('"Cancel"');
  process.exit(0);
}

process.stdout.write('"Continue"');
