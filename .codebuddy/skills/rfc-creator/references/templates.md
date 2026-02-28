# RFC Templates

Complete templates for creating RFC documents.

## Full RFC Template

```markdown
# RFC NNNN: [Title]

> **状态**: Draft
> **作者**: [author]
> **创建日期**: [YYYY-MM-DD]
> **目标版本**: v[X.Y.Z]

## 摘要

[One paragraph summary of the proposal. What is being proposed and why?]

## 动机

### 当前状态分析

[Describe the current state and its limitations]

现有功能/配置：

```toml
# Current format example
[section]
field = "value"
```

### 行业趋势对比

| 工具 | 特点 | 可借鉴 |
|------|------|--------|
| **Tool A** | Feature description | What we can learn |
| **Tool B** | Feature description | What we can learn |
| **Tool C** | Feature description | What we can learn |

### 需求分析

1. **需求 1** - 描述为什么需要这个功能
2. **需求 2** - 描述为什么需要这个功能
3. **需求 3** - 描述为什么需要这个功能

## 设计方案

### 完整配置/API 预览

```toml
# Proposed format - complete example
[section]
new_field = "value"
enhanced_field = { key = "value", option = true }

[new_section]
feature_a = true
feature_b = "config"

[new_section.nested]
option_1 = "value"
option_2 = ["list", "of", "values"]
```

### 详细说明

#### 1. Feature A

[Detailed explanation of Feature A]

**配置示例：**

```toml
[feature_a]
enabled = true
option = "value"
```

**字段说明：**

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `enabled` | bool | `false` | 是否启用 |
| `option` | string | `"default"` | 配置选项 |

#### 2. Feature B

[Detailed explanation of Feature B]

**配置示例：**

```toml
[feature_b]
mode = "auto"
items = ["item1", "item2"]
```

**支持的模式：**

- `auto` - 自动检测
- `manual` - 手动配置
- `disabled` - 禁用

### 命令行支持

```bash
# New commands
tool feature-a --option value
tool feature-b list

# Updated commands
tool existing-cmd --new-flag
```

### 错误处理

| 错误场景 | 错误信息 | 建议操作 |
|----------|----------|----------|
| Invalid config | "Invalid value for field X" | Check documentation |
| Missing dependency | "Required Y not found" | Install Y first |

## 向后兼容性

### 兼容策略

1. **版本检测**: 通过 `min_version` 字段检测配置版本
2. **渐进增强**: 所有新字段都是可选的
3. **默认值**: 新字段都有合理的默认值
4. **警告提示**: 遇到未知字段时发出警告而非报错

### 破坏性变更

[List any breaking changes, or state "None"]

### 迁移路径

```bash
# 检查配置兼容性
tool config check

# 自动迁移
tool config migrate --to v2

# 验证迁移结果
tool config validate
```

### 迁移示例

**迁移前 (v1):**

```toml
[old_section]
old_field = "value"
```

**迁移后 (v2):**

```toml
[new_section]
new_field = "value"
enhanced = true
```

## 实现计划

### Phase 1: 核心功能 (vX.Y.0)

- [ ] Feature A 基础实现
- [ ] Feature B 基础实现
- [ ] 配置验证
- [ ] 迁移工具

### Phase 2: 扩展功能 (vX.Y+1.0)

- [ ] Feature A 高级选项
- [ ] Feature B 高级选项
- [ ] 性能优化

### Phase 3: 完善 (vX.Y+2.0)

- [ ] 边缘情况处理
- [ ] 文档完善
- [ ] 社区反馈整合

## 替代方案

### 方案 A: [Alternative approach name]

[Description of alternative and why it was not chosen]

**优点:**
- Pro 1
- Pro 2

**缺点:**
- Con 1
- Con 2

### 方案 B: [Another alternative]

[Description and reasoning]

## 安全考虑

[Security implications of this proposal, if any]

- 考虑点 1
- 考虑点 2

## 参考资料

- [相关工具文档](url)
- [行业标准](url)
- [内部设计文档](path)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| YYYY-MM-DD | Draft | 初始草案 |
```

## Implementation Tracker Template

```markdown
# RFC NNNN: Implementation Tracker

> 关联 RFC: [RFC NNNN: Title](./NNNN-title.md)

## 总体进度

| Phase | 状态 | 完成度 | 目标版本 |
|-------|------|--------|----------|
| Phase 1: 核心功能 | ⏳ 进行中 | 40% | vX.Y.0 |
| Phase 2: 扩展功能 | ⏸️ 待开始 | 0% | vX.Y+1.0 |
| Phase 3: 完善 | ⏸️ 待开始 | 0% | vX.Y+2.0 |

## 详细进度

### Phase 1: 核心功能

#### 1.1 Feature A

| 任务 | 状态 | 负责人 | PR |
|------|------|--------|-----|
| 设计文档 | ✅ 完成 | @author | - |
| 核心实现 | ✅ 完成 | @author | #123 |
| 单元测试 | ⏳ 进行中 | @author | - |
| 集成测试 | ⏸️ 待开始 | - | - |
| 文档 | ⏸️ 待开始 | - | - |

**实现文件:**
- `crates/module/src/feature_a.rs`
- `crates/module/src/feature_a/mod.rs`

#### 1.2 Feature B

| 任务 | 状态 | 负责人 | PR |
|------|------|--------|-----|
| 设计文档 | ✅ 完成 | @author | - |
| 核心实现 | ⏸️ 待开始 | - | - |
| 单元测试 | ⏸️ 待开始 | - | - |
| 集成测试 | ⏸️ 待开始 | - | - |
| 文档 | ⏸️ 待开始 | - | - |

### Phase 2: 扩展功能

[To be planned after Phase 1 completion]

### Phase 3: 完善

[To be planned after Phase 2 completion]

## 测试计划

### 单元测试

- [ ] `tests/feature_a_tests.rs` - Feature A 测试
- [ ] `tests/feature_b_tests.rs` - Feature B 测试
- [ ] `tests/migration_tests.rs` - 迁移测试

### 集成测试

- [ ] Feature A + Feature B 集成
- [ ] 向后兼容性测试
- [ ] 配置验证测试

### E2E 测试

- [ ] 完整工作流测试
- [ ] 性能基准测试
- [ ] 边缘情况测试

## 文档更新

- [ ] `docs/config/reference.md` - 配置参考
- [ ] `docs/guide/feature-guide.md` - 功能指南
- [ ] `docs/guide/migration.md` - 迁移指南
- [ ] `docs/guide/best-practices.md` - 最佳实践

## 已知问题

| 问题 | 严重程度 | 状态 | 备注 |
|------|----------|------|------|
| Issue 1 | 中 | 待解决 | 描述 |

## 更新日志

| 日期 | 变更 |
|------|------|
| YYYY-MM-DD | 创建实现跟踪文档 |
| YYYY-MM-DD | 完成 Feature A 核心实现 |
```

## Minimal RFC Template

For smaller proposals:

```markdown
# RFC NNNN: [Title]

> **状态**: Draft
> **作者**: [author]
> **创建日期**: [YYYY-MM-DD]

## 摘要

[Brief summary]

## 动机

[Why is this needed?]

## 设计方案

[Technical details]

```code
# Example
```

## 实现计划

- [ ] Task 1
- [ ] Task 2
- [ ] Task 3

## 更新记录

| 日期 | 变更 |
|------|------|
| YYYY-MM-DD | 初始草案 |
```
