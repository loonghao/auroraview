基于我们当前的修改添加对应的文档，需要添加中英双语文档。
需要通俗易懂的描述我们的新功能，以及如何使用。还有一些扩展用法等，更新后提交到远端。

## VitePress 文档规范

### 链接格式

VitePress 内部链接**不要**使用 `.md` 扩展名：

```markdown
# 正确
- [Qt 集成](./qt-integration)
- [Maya 集成](../dcc/maya)
- [DCC 概览](../dcc/)

# 错误 - 会导致 dead link 错误
- [Qt 集成](./qt-integration.md)
- [Maya 集成](../dcc/maya.md)
- [DCC 概览](../dcc/index.md)
```

### 目录索引链接

引用目录的 `index.md` 时，使用目录路径加斜杠：

```markdown
# 正确
[DCC 概览](../dcc/)

# 错误
[DCC 概览](../dcc/index.md)
[DCC 概览](../dcc/index)
```

### 中英文文档同步

- 英文文档：`docs/guide/*.md`、`docs/dcc/*.md`
- 中文文档：`docs/zh/guide/*.md`、`docs/zh/dcc/*.md`
- 两者内容必须保持一致，仅语言不同
