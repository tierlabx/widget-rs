# Widget-RS CLI (`widget-cli`)

`widget-cli` 是 `widget-rs` 的辅助构建、管理与自动化发布工具。作为一个独立的命令行工具，它主要用于简化插件管理、自动化版本计算发布、维护更新日志（CHANGELOG）并与 CI/CD 流水线联动。

---

## 核心功能

1. **自动化版本发布 (`release`)**：基于 [Conventional Commits](https://www.conventionalcommits.org/) 约定式提交规范，自动分析 Git 历史、计算下一语义化版本号、同步更新 `Cargo.toml` 与 `Cargo.lock`、生成 `CHANGELOG.md` 并提交与推送 Git Tag。
2. **源码级插件管理 (`plugin`)**：一键注入或移除插件依赖与源码注册表，实现无缝插件扩展。
3. **CI/CD 自动化联动**：本地发版推送 Tag 后，自动触发 GitHub Actions 打包 Windows 安装包（NSIS / WiX MSI）并创建 GitHub Release。

---

## 命令行使用指南

### 1. 版本发布 (Release)

```bash
# 自动根据自上次发版以来的 commit 历史计算版本号并完成发布
cargo run -p widget-cli -- release

# 预览下一个版本号及 CHANGELOG 内容（dry-run 模式，不做任何实际修改）
cargo run -p widget-cli -- release --dry-run

# 手动指定目标版本号发布
cargo run -p widget-cli -- release --version 0.7.0
```

#### 版本号晋升 (Bump) 规则

`widget-cli` 根据 commit 消息前缀自动确定语义化版本（SemVer: `MAJOR.MINOR.PATCH`）的晋升级别：

| 提交前缀 / 标记 | 含义说明 | 版本晋升级别 | CHANGELOG 分组 |
| :--- | :--- | :--- | :--- |
| 包含 `BREAKING CHANGE` 或前缀带 `!` (如 `feat!:`) | 破坏性重大变更 | **MAJOR** (`+1.0.0`) | `Changed` |
| `feat:` / `feat(scope):` | 新功能 / 新特性 | **MINOR** (`0.+1.0`) | `Added` |
| `fix:` / `fix(scope):` | Bug 修复 | **PATCH** (`0.0.+1`) | `Fixed` |
| `perf:` / `perf(scope):` | 性能优化 | **PATCH** (`0.0.+1`) | `Fixed` |
| `refactor:` / `refactor(scope):` | 代码重构 | **PATCH** (`0.0.+1`) | `Changed` |
| `chore:` / `docs:` / `ci:` / `style:` / `test:` | 构建配置、文档、测试等 | **None**（不触发版本号升级） | `Skip`（不计入日志） |
| 未遵循格式的其他提交 | 普通提交 | **PATCH** (`0.0.+1`) | `Other` |

> **多提交晋升原则**：若发布周期内包含多条提交，取**最高优先级**生效（`MAJOR > MINOR > PATCH`）。例如：包含 1 条 `feat` 和 2 条 `fix` 时，版本将按 `feat` 升级 **MINOR** 版本（如 `v0.6.1` -> `v0.7.0`）。

#### 发版自动化执行步骤

1. 扫描当前分支最新 Tag 后的所有提交并计算新版本号。
2. 批量更新工作区根目录及所有子 crate / plugin 的 `Cargo.toml` 版本字段。
3. 执行 `cargo check` 自动同步 `Cargo.lock`。
4. 在 `CHANGELOG.md` 顶部生成格式化更新条目。
5. 执行 `git commit -m "chore: release vX.Y.Z"` 提交变更。
6. 创建本地 Git Tag（如 `v0.7.0`）。
7. 执行 `git push origin <branch> --tags` 推送至远程仓库。

---

### 2. 插件管理 (Plugin)

`widget-cli` 支持在源码级别快速安装与卸载小部件插件：

```bash
# 添加本地插件（自动将依赖注入 crates/app/Cargo.toml 并注册至 plugin_registry.rs）
cargo run -p widget-cli -- plugin add <插件名称> --path <插件本地路径>
# 示例：
cargo run -p widget-cli -- plugin add custom_widget --path ../plugins/custom_widget

# 卸载已安装的插件
cargo run -p widget-cli -- plugin remove <插件名称>
# 示例：
cargo run -p widget-cli -- plugin remove custom_widget
```

---

## CI/CD 打包与发布联动

当 `widget-cli release` 推送新的 `v*` Tag 后，GitHub Actions 自动化流水线（`.github/workflows/packager.yml`）将自动触发：

1. **环境准备**：基于 `windows-latest` 安装 Rust 稳定版工具链、`cargo-packager` 及 WiX Toolset。
2. **打包构筑**：执行 `cargo packager --release -f nsis -f wix` 生成安装包：
   - NSIS 安装程序：`target/release/*-setup.exe`
   - WiX MSI 安装包：`target/release/*.msi`
3. **GitHub Release**：自动创建 GitHub Release，挂载安装包资产并同步 Release Notes。
