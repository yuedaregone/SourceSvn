# 测试策略

## 测试层级

```
单元测试 (60%)
├── 前端：Vitest
├── 后端：cargo test
集成测试 (30%)
├── Tauri 命令测试
├── SVN 命令集成测试
E2E 测试 (10%)
└── Playwright (MVP 可选)
```

## 后端测试

### 单元测试

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

**覆盖范围**：
- SVN XML 解析函数（`parse_status_xml`, `parse_log_xml`, `parse_list_xml`, `parse_info_xml`, `parse_update_xml`）
- 提交 revision 提取（`extract_revision_from_output`）
- 配置读写与迁移
- Shelve 路径计算与名称校验

**现有测试（12 个）**：
- `svn/status.rs`：3 个（单文件、多文件、空目标）
- `svn/log.rs`：2 个（单条日志含路径、空日志）
- `svn/commit.rs`：2 个（revision 提取成功/失败）
- `svn/list.rs`：1 个（目录列表含文件和目录）
- `svn/info.rs`：2 个（正常解析、无 entry 错误）
- `svn/update.rs`：2 个（含修订号更新、无变更更新）

**示例**：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_xml_modified() {
        let xml = r#"<?xml version="1.0"?>
<status>
  <target>
    <entry path="src/main.rs">
      <wc-status item="modified" revision="100"/>
    </entry>
  </target>
</status>"#;
        let result = parse_status_xml(xml).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "src/main.rs");
        assert_eq!(result[0].status, FileStatusType::Modified);
    }
}
```

### 集成测试

使用 `tests/fixtures/` 下的测试仓库：

```bash
# 准备测试仓库
tests/fixtures/setup.sh

# 运行集成测试
cargo test --test integration -- --ignored
```

## 前端测试

### 单元测试 (Vitest)

```bash
pnpm test:unit
```

**覆盖范围**：
- Pinia store actions
- 工具函数（日期格式化、diff 解析）
- 组件渲染（基础）

### 组件测试

```bash
pnpm test:component
```

## 覆盖率目标

| 层级 | 目标 |
|------|------|
| 后端核心模块 | ≥80% |
| 前端工具函数 | ≥70% |
| 前端组件 | ≥50% (MVP 阶段) |

## CI 集成 (GitHub Actions)

```yaml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 22 }
      - uses: actions-rs/toolchain@v1
        with: { toolchain: stable }
      - run: pnpm install
      - run: pnpm test:unit
      - run: cargo test --manifest-path src-tauri/Cargo.toml
```

## 本地测试仓库准备

```bash
# 一键创建测试环境
tests/fixtures/create_test_repo.sh
```
