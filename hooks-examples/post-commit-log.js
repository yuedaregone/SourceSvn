#!/usr/bin/env node
// PostCommit hook - 提交后记录日志
// 将提交信息追加到指定日志文件
// 用法：在 hooks.toml 中配置 script_path 指向此文件

const fs = require("fs");
const path = require("path");

const context = JSON.parse(process.argv[2]);
const { hook_type, repo_path, data, timestamp } = context;
const { revision, message, author, files } = data;

const logDir = path.join(process.env.HOME || process.env.USERPROFILE, ".sourcesvn", "logs");
const logFile = path.join(logDir, "commit-history.log");

try {
  if (!fs.existsSync(logDir)) {
    fs.mkdirSync(logDir, { recursive: true });
  }

  const entry = [
    `[${timestamp}] r${revision} by ${author}`,
    `  Message: ${message}`,
    `  Files: ${(files || []).join(", ")}`,
    "",
  ].join("\n");

  fs.appendFileSync(logFile, entry + "\n");
} catch (err) {
  console.error(`[PostCommit] 写入日志失败: ${err.message}`);
}
