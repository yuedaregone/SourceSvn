# 代码规范

## 前端 (TypeScript + Vue 3)

### 命名规范

| 类型 | 规则 | 示例 |
|------|------|------|
| 组件文件 | PascalCase | `LogView.vue` |
| 普通 TS 文件 | kebab-case | `svn-service.ts` |
| 变量/函数 | camelCase | `getRepoInfo()` |
| 常量 | UPPER_SNAKE_CASE | `MAX_RETRY_COUNT` |
| 类型/接口 | PascalCase | `FileStatus` |
| 枚举 | PascalCase | `FileStatusType` |
| 枚举成员 | UPPER_SNAKE_CASE 或 PascalCase | `Modified` |

### ESLint 配置 (eslint.config.js)

```javascript
import js from '@eslint/js'
import vue from 'eslint-plugin-vue'
import ts from 'typescript-eslint'

export default ts.config(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...vue.configs['flat/recommended'],
  {
    rules: {
      'no-console': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/explicit-function-return-type': 'off',
      'vue/multi-word-component-names': 'off',
      'vue/require-default-prop': 'off',
    },
  }
)
```

### Prettier 配置 (.prettierrc)

```json
{
  "semi": false,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5",
  "printWidth": 100,
  "endOfLine": "auto"
}
```

## 后端 (Rust)

### 命名规范

| 类型 | 规则 | 示例 |
|------|------|------|
| 变量/函数 | snake_case | `run_svn()` |
| 类型/Trait | PascalCase | `FileStatus` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_TIMEOUT_SECS` |
| 模块 | snake_case | `svn_service` |

### rustfmt 配置 (rustfmt.toml)

```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 2
use_small_heuristics = "Max"
reorder_imports = true
reorder_modules = true
```

### Clippy 规则

- 禁止 `unwrap()` 在生产代码（测试除外）
- 使用 `expect()` 或 `?` 替代
- `cargo clippy -- -D warnings`

## Git 提交规范

格式：`<type>(<scope>): <subject>`

| type | 说明 |
|------|------|
| feat | 新功能 |
| fix | 修复 |
| docs | 文档 |
| style | 格式（不影响逻辑） |
| refactor | 重构 |
| test | 测试 |
| chore | 构建/工具 |

示例：`feat(svn): add svn_log command with XML parsing`
