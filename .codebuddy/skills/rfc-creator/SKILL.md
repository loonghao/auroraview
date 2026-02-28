---
name: rfc-creator
description: |
  This skill helps create RFC (Request for Comments) documents for proposing new features,
  architectural changes, or significant enhancements to the project. It provides templates,
  structure guidelines, and best practices for writing effective technical proposals.
  Use this skill when planning major changes that need team review and discussion.
---

# RFC Creator

This skill guides the creation of RFC (Request for Comments) documents for technical proposals.

## When to Use

- Proposing a new feature or capability
- Planning architectural changes
- Introducing breaking changes
- Major refactoring proposals
- New configuration formats or APIs
- Integration with external systems

## RFC Document Structure

### Standard Sections

1. **Header** - Metadata (status, author, date, target version)
2. **摘要/Summary** - Brief overview of the proposal
3. **动机/Motivation** - Why this change is needed
4. **设计方案/Design** - Detailed technical design
5. **向后兼容性/Backward Compatibility** - Migration and compatibility considerations
6. **实现计划/Implementation Plan** - Phased implementation roadmap
7. **参考资料/References** - Related documents and resources
8. **更新记录/Changelog** - Document revision history

## Step 1: Create RFC Directory

```
docs/rfcs/
├── NNNN-short-title.md           # Main RFC document
└── NNNN-implementation-tracker.md # Implementation progress tracker (optional)
```

RFC numbers are assigned sequentially: `0001`, `0002`, etc.

## Step 2: RFC Header Template

```markdown
# RFC NNNN: Title

> **状态**: Draft | Review | Accepted | Implemented | Rejected
> **作者**: author name/team
> **创建日期**: YYYY-MM-DD
> **目标版本**: vX.Y.Z
```

### Status Definitions

| Status | Description |
|--------|-------------|
| **Draft** | Initial proposal, open for major changes |
| **Review** | Ready for team review and feedback |
| **Accepted** | Approved for implementation |
| **Implemented** | Fully implemented and released |
| **Rejected** | Not accepted (with documented reasons) |

## Step 3: Write the RFC Content

### 3.1 Summary Section

```markdown
## 摘要

[One paragraph describing what this RFC proposes and its main benefits]
```

### 3.2 Motivation Section

```markdown
## 动机

### 当前状态分析
[Describe current limitations or problems]

### 行业趋势对比
[Compare with similar tools/approaches in the industry]

| 工具 | 特点 | 可借鉴 |
|------|------|--------|
| Tool A | Feature X | Learning Y |

### 需求分析
[List specific requirements this proposal addresses]

1. **Requirement 1** - Description
2. **Requirement 2** - Description
```

### 3.3 Design Section

```markdown
## 设计方案

### 完整配置/API 预览

```toml/yaml/json
# Complete example of the proposed format
```

### 详细说明

#### Section 1: Feature Name
[Detailed explanation with examples]

#### Section 2: Feature Name
[Detailed explanation with examples]
```

### 3.4 Backward Compatibility Section

```markdown
## 向后兼容性

### 兼容策略

1. **Version Detection** - How to detect old vs new format
2. **Gradual Enhancement** - All new fields are optional
3. **Default Values** - Sensible defaults for new fields
4. **Warning Handling** - Warn on unknown fields, don't error

### 迁移路径

```bash
# Check compatibility
command check

# Auto-migrate
command migrate --to v2

# Validate
command validate
```
```

### 3.5 Implementation Plan Section

```markdown
## 实现计划

### Phase 1: Core Features (vX.Y.0)

- [ ] Feature A
- [ ] Feature B
- [ ] Migration tooling

### Phase 2: Extended Features (vX.Y+1.0)

- [ ] Feature C
- [ ] Feature D

### Phase 3: Advanced Features (vX.Y+2.0)

- [ ] Feature E
- [ ] Feature F
```

### 3.6 References Section

```markdown
## 参考资料

- [Related Tool Documentation](url)
- [Industry Standard](url)
- [Internal Design Doc](path)
```

### 3.7 Changelog Section

```markdown
## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| YYYY-MM-DD | Draft | 初始草案 |
| YYYY-MM-DD | Review | 根据反馈更新 |
```

## Step 4: Create Implementation Tracker (Optional)

For complex RFCs, create a separate tracker document:

```markdown
# RFC NNNN: Implementation Tracker

## 总体进度

| Phase | 状态 | 完成度 | 目标版本 |
|-------|------|--------|----------|
| Phase 1 | 进行中 | 60% | vX.Y.0 |
| Phase 2 | 待开始 | 0% | vX.Y+1.0 |

## 详细进度

### Phase 1: Core Features

#### Feature A
- [x] Design
- [x] Implementation
- [ ] Tests
- [ ] Documentation

#### Feature B
- [ ] Design
- [ ] Implementation
- [ ] Tests
- [ ] Documentation

## 测试计划

### 单元测试
- [ ] Test case 1
- [ ] Test case 2

### 集成测试
- [ ] Integration test 1
- [ ] Integration test 2

### E2E 测试
- [ ] E2E test 1
- [ ] E2E test 2

## 文档更新

- [ ] Config reference
- [ ] User guide
- [ ] Migration guide
- [ ] Best practices

## 更新日志

| 日期 | 变更 |
|------|------|
| YYYY-MM-DD | 创建跟踪文档 |
```

## Best Practices

### Writing Effective RFCs

1. **Be Specific** - Include concrete examples and code snippets
2. **Consider Edge Cases** - Address error handling and unusual scenarios
3. **Think About Migration** - Always plan for existing users
4. **Keep It Focused** - One RFC per major feature/change
5. **Iterate** - RFCs can be updated based on feedback

### Code Examples

- Use realistic, working examples
- Show both simple and advanced usage
- Include error cases where relevant

### Tables and Diagrams

- Use tables for comparisons and status tracking
- Include ASCII diagrams for architecture when helpful
- Keep formatting consistent

### Review Process

1. Share RFC with team for initial feedback
2. Address comments and update document
3. Move to "Review" status when ready
4. Get formal approval before implementation
5. Update status as implementation progresses

## RFC Naming Convention

```
NNNN-short-descriptive-title.md
```

Examples:
- `0001-config-v2-enhancement.md`
- `0002-plugin-architecture.md`
- `0003-remote-development-support.md`

## Reference Templates

See `references/templates.md` for complete RFC templates.
