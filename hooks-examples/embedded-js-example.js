// 嵌入式JS引擎示例 - 使用内置API
// 此脚本由SourceSvn内置JS引擎执行，可直接调用应用功能

// 获取提交信息
var message = (context.data.message || "").trim();
var files = context.data.files || [];
var author = context.data.author || "unknown";

// 显示Toast通知
toast("info", "正在验证提交: " + message);

// 检查提交信息格式
var VALID_TYPES = ["feat", "fix", "docs", "style", "refactor", "test", "chore", "perf", "ci", "build", "revert"];
var pattern = new RegExp("^(" + VALID_TYPES.join("|") + ")(\\(.+\\))?: .+");

if (!message) {
  toast("error", "提交信息不能为空");
  cancel("empty message");
}

if (!pattern.test(message)) {
  toast("error", "提交信息格式错误，要求: type: description");
  cancel("bad format");
}

// 记录日志
log("info", "提交验证通过: " + message + " by " + author);
log("info", "修改文件: " + files.join(", "));

// 可以调用SVN操作获取更多信息
// var status = svnStatus(context.repo_path);
// var info = svnInfo(context.repo_path);
// var logEntries = svnLog(context.repo_path, 5);

toast("success", "提交验证通过");
