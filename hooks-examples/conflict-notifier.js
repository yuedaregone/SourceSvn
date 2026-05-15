#!/usr/bin/env node
// ConflictDetected hook - 冲突通知
// 检测到冲突时发送通知（示例：写入桌面通知文件）
// 用法：在 hooks.toml 中配置 script_path 指向此文件

const fs = require("fs");
const path = require("path");

const context = JSON.parse(process.argv[2]);
const { repo_path, data } = context;
const files = data.files || [];
const conflictType = data.conflict_type || "unknown";

const desktop = path.join(process.env.HOME || process.env.USERPROFILE, "Desktop");
const notifyFile = path.join(desktop, `svn-conflict-${Date.now()}.txt`);

const content = [
  "SVN 冲突通知",
  "============",
  "",
  `仓库: ${repo_path}`,
  `冲突类型: ${conflictType}`,
  `冲突文件:`,
  ...files.map(f => `  - ${f}`),
  "",
  `时间: ${new Date().toLocaleString()}`,
  "",
  "请打开 SourceSvn 解决冲突。",
].join("\n");

try {
  fs.writeFileSync(notifyFile, content);
  console.error(`[ConflictDetected] 冲突通知已写入: ${notifyFile}`);
} catch (err) {
  console.error(`[ConflictDetected] 写入通知失败: ${err.message}`);
}
