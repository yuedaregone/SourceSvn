# 多语言资源目录

此目录用于存放 SourceSvn 的多语言翻译文件。

## 目录结构

```
i18n/
├── README.md           # 本文件
├── locales/            # 语言包
│   ├── zh-CN.json      # 简体中文
│   ├── en-US.json      # 英语
│   └── ...
└── scripts/            # 辅助脚本（可选）
    └── validate.js     # 验证 JSON 文件完整性
```

## 语言文件格式

所有语言文件遵循相同的 JSON 结构，键名采用点分隔的层级结构。

## 如何添加新语言

1. 复制 `en-US.json` 到 `xx-YY.json`（语言-国家/地区）
2. 翻译所有字段的值，保留键名
3. 在 `src/i18n/index.ts` 中导入并注册
4. 在配置中添加语言选项（`src/views/SettingsPage.vue`）

## 验证语言文件

```bash
node scripts/validate.js locales/zh-CN.json
```

## 注意事项

- 所有语言文件必须包含完全相同的键集合，缺少的键会导致 fallback 语言失败
- 支持插值变量 `{variableName}` (如 `"Hello {name}"`)
- 保持界面文本的简洁和一致性
