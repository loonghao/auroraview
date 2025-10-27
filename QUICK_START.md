# AuroraView 快速开始指南

## 🚀 5分钟快速开始

### 前置要求
- Rust 1.75+
- Python 3.7+
- Git

### 1. 克隆仓库
```bash
git clone https://github.com/loonghao/auroraview.git
cd auroraview
```

### 2. 设置开发环境
```bash
# 使用justfile（推荐）
just dev

# 或者手动设置
cargo build --release
pip install -e .
```

### 3. 运行测试
```bash
# 运行所有测试
just test

# 或者
cargo test --lib
pytest tests/ -v
```

### 4. 查看示例
```bash
# 运行示例
python examples/simple_window.py
```

## 📖 常用命令

### 开发命令
```bash
just build          # 构建扩展
just test           # 运行所有测试
just format         # 格式化代码
just lint           # 检查代码
just clean          # 清理构建产物
```

### 测试命令
```bash
just test-fast      # 快速测试
just test-unit      # 单元测试
just test-cov       # 覆盖率测试
just test-file FILE # 运行特定文件
```

### CI命令
```bash
just ci-build       # CI构建
just ci-lint        # CI检查
just ci-test-rust   # Rust测试
just ci-test-python # Python测试
```

## 🔧 项目结构

```
auroraview/
├── src/                    # Rust源代码
│   ├── lib.rs             # 主模块
│   ├── utils/             # 工具函数
│   └── webview/           # WebView实现
├── python/                # Python绑定
│   └── auroraview/        # Python包
├── tests/                 # 测试
│   ├── test_basic.py      # 基础测试
│   ├── test_webview.py    # WebView测试
│   ├── test_decorators.py # 装饰器测试
│   └── test_integration.py # 集成测试
├── examples/              # 示例代码
├── docs/                  # 文档
├── Cargo.toml            # Rust配置
├── pyproject.toml        # Python配置
├── justfile              # 开发命令
└── README.md             # 项目说明
```

## 📝 编写代码

### Rust代码
```rust
// 遵循Rust编码规范
// 使用cargo fmt格式化
// 使用cargo clippy检查

cargo fmt --all
cargo clippy --all-targets --all-features
```

### Python代码
```python
# 遵循PEP 8规范
# 使用ruff格式化
# 使用ruff检查

ruff format python/ tests/
ruff check python/ tests/
```

## 🧪 编写测试

### Rust测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }
}
```

### Python测试
```python
import pytest

@pytest.mark.unit
def test_something():
    assert 2 + 2 == 4

@pytest.mark.integration
def test_integration():
    # 集成测试
    pass
```

## 🐛 调试

### Rust调试
```bash
# 启用日志
RUST_LOG=debug cargo test

# 使用调试器
rust-gdb target/debug/auroraview_core
```

### Python调试
```bash
# 启用日志
python -c "import logging; logging.basicConfig(level=logging.DEBUG)"

# 使用pdb
python -m pdb examples/simple_window.py
```

## 📦 发布

### 构建轮子
```bash
just release
```

### 发布到PyPI
```bash
# 需要配置PyPI凭证
just publish
```

## 🤝 贡献

1. Fork项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启Pull Request

### 提交规范
遵循 [Conventional Commits](https://www.conventionalcommits.org/)：
- `feat: 新功能`
- `fix: 修复bug`
- `docs: 文档更新`
- `style: 代码风格`
- `refactor: 代码重构`
- `test: 测试相关`
- `chore: 构建相关`

## 📚 更多资源

- [完整文档](./docs/)
- [API参考](./docs/TECHNICAL_DESIGN.md)
- [DCC集成指南](./docs/DCC_INTEGRATION_GUIDE.md)
- [与PyWebView的对比](./docs/COMPARISON_WITH_PYWEBVIEW.md)

## ❓ 常见问题

### Q: 如何安装依赖？
A: 使用 `just install` 或 `pip install -e .`

### Q: 如何运行测试？
A: 使用 `just test` 或 `pytest tests/ -v`

### Q: 如何格式化代码？
A: 使用 `just format`

### Q: 如何检查代码质量？
A: 使用 `just lint`

### Q: 如何生成覆盖率报告？
A: 使用 `just test-cov`

## 📞 联系方式

- 作者: Hal Long
- 邮箱: hal.long@outlook.com
- GitHub: [@loonghao](https://github.com/loonghao)

---

**祝你开发愉快！** 🎉

