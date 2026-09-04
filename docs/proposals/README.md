# Plugins 规划与孵化体系 (Plugin Proposals)

本目录用于存放 `widget-rs` 新插件/小组件的功能设计、技术架构预研与实施规划文档，作为新组件从概念到工程落地的统一设计中心。

---

## 1. 提案生命周期 (Proposal Lifecycle)

每个新组件或重构提案按以下阶段推进：

```
[ 1. 概念与提议 (Draft) ] 
       ↓ 
[ 2. 方案评审与技术验证 (RFC / Review) ] 
       ↓ 
[ 3. 核心原型开发 (Prototype) ] 
       ↓ 
[ 4. 正式插件工程落地 (Implementation in plugins/) ] 
       ↓ 
[ 5. 稳定发布 (Release) ]
```

- **Draft**：阐述组件背景、目标痛点、用户交互形态及预期功能点。
- **Review / RFC**：给出详细的技术架构、渲染路径、数据流、状态机以及与 `widget-core` 的集成规范，确保无重复造轮子且符合系统约束。
- **Prototype**：在独立模块或最小实验中验证关键技术可行性（如 Windows API 交互、GPUI 复杂着色器/绘制管线）。
- **Implementation**：正式通过 `widget-cli new-plugin` 脚手架在 `plugins/` 下生成标准 crate 并接入。

---

## 2. 提案目录规范

所有新增小部件规划统一在 `docs/proposals/<plugin-slug>/` 建立独立子目录，并在本文件进行索引登记。

### 目录推荐结构
```text
docs/proposals/<plugin-slug>/
├── README.md                          # 提案总览、定位与实施路线图 (Milestones)
├── architecture-and-rendering.md      # UI 与 GPUI 渲染管线、动效设计方案
├── smtc-and-lyrics-engine.md          # 核心业务引擎与外部协议交互（根据插件类型选配）
└── window-and-integration.md          # 窗口宿主交互、配置持久化与核心规范遵循
```

---

## 3. 严格遵循的设计与编码红线

在起草与实现新插件提案时，必须严格遵守以下规范：

1. **桌面常驻保护 (Win+D 常驻)**：
   - 必须在窗口生成时挂载至系统 `Progman`，用户按 `Win+D` 时小组件必须保留在桌面上，不能随普通应用最小化。
2. **容器规范封装**：
   - 小部件窗口必须统一使用 `widget_core::WidgetWindow<T>` 容器包装。
   - 严禁在插件内手动编写编辑模式检测、拖拽手柄、窗口边框切换逻辑。
3. **独立设置窗口规范**：
   - 设置弹窗必须使用 `widget_core::render_settings_shell`，并复用 `settings_card` 与 `settings_section_header`。
4. **单文件与性能红线**：
   - 任何 Rust 源文件不能超过 400 行，业务逻辑必须模块化拆分。
   - 渲染必须保证 60Hz/120Hz 垂直同步，内存占用控制在 50MB 以内。
5. **UI 页面禁用 Emoji**：
   - 小组件界面呈现中禁止包含 emoji 图标，统一采用矢量路径图标或系统符号。

---

## 4. 当前规划中的组件清单 (Active Proposals)

| 提案编号 | 组件名称 | 目录链接 | 当前阶段 | 核心特色 |
| :--- | :--- | :--- | :--- | :--- |
| **PROP-001** | **沉浸式桌面流光歌词 (Folia Lyrics)** | [folia-lyrics](./folia-lyrics/README.md) | **Review / RFC** | 借鉴 Folia-Major，SMTC 全局媒体监听 + 逐字卡拉OK光效 + 流光动态舞台 + 桌面常驻 |
