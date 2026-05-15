#!/usr/bin/env node
// PreUpdate hook - 更新前检查
// 如果工作副本有未提交的修改，阻止更新以防冲突
// 用法：在 hooks.toml 中配置 script_path 指向此文件

const { execSync } = require("child_process");

const context = JSON.parse(process.argv[2]);
const { repo_path } = context;

try {
  const output = execSync("svn status -q", {
    cwd: repo_path,
    encoding: "utf-8",
    timeout: 10000,
  }).trim();

  if (output.length > 0) {
    const lines = output.split("\n").length;
    console.error(`[PreUpdate] 工作副本有 ${lines} 个未提交的修改，请先提交或还原`);
    process.stdout.write('"Cancel"');
    process.exit(0);
  }
} catch (err) {
  console.error(`[PreUpdate] 检查状态失败: ${err.message}`);
  // 检查失败不阻止更新，继续执行
}

process.stdout.write('"Continue"');
