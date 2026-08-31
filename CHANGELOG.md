# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.7.0](https://github.com/tierlabx/widget-rs/releases/tag/v0.7.0) - 2026-08-31

### Added

- 支持启动自动检查更新、分块下载进度反馈、更新弹窗并重构设置页面布局

### Fixed

- 修复fences 组件终端bug

## [0.6.1](https://github.com/tierlabx/widget-rs/releases/tag/v0.6.1) - 2026-08-31

### Fixed

- 优化自适应心跳机制降低能耗，重构模块化架构并补充单元测试

## [0.6.0](https://github.com/tierlabx/widget-rs/releases/tag/v0.6.0) - 2026-08-31

### Added

- 优化多屏显示准确度、无缝边缘吸附与网格相同高度自动对齐
- 支持卡片拖拽排序、紧凑化布局及悬停显示关闭按钮
- 添加分类标签编辑与删除功能并完成模块化重构

### Fixed

- 修复多屏DPI副屏窗口反复放大Bug并实现Stretchly展开高度自适应
- 修复系统托盘切换控制面板失效问题并支持双击与状态动态同步
- fences 高度适应修复
- 在 release 发布流程中自动刷新 Cargo.lock 并纳入版本提交，杜绝遗漏

## [0.5.0](https://github.com/tierlabx/widget-rs/releases/tag/v0.5.0) - 2026-08-28

### Added

- 将分类标签重构为左侧纵向突出吸附 Tab 侧边栏，支持胶囊状态高亮与一键添加标签
- 推出精准定时与周期循环提醒引擎，新增横向分类标签导航与聚焦过滤
- 支持项目管理甘特色系，待办列表与通知提醒新增移入展开更多详情面板
- 重构为程序、文件夹、文件三栏可折叠桌面收纳盒架构，支持智能拖拽归类与独立快速添加
- 默认内容清空为纯净初始态，并全面支持桌面文件/文件夹拖拽收纳与新分类创建
- 桌面小组件全面升级，新增 Fences 收纳、Sticky 多便签、Todo 悬浮胶囊及透明白底修复

### Fixed

- 移除默认写死的示例待办，初始状态干净清爽并增加空状态提示
- 在删除按钮的 on_mouse_down 与 on_click 均添加 stop_propagation，彻底阻断事件冒泡打开文件
- 优化条目卡片删除按钮视觉显色，并将启动触发区与删除操作彻底解耦，防止误触打开
- 修复便签输入文字颜色为深墨水黑，并移除右上角冗余的文件按钮（支持直接拖拽图片）
- 切换为深色模式全局主题，解决 Input 输入框文字在深色背景下呈黑色看不见的问题

### Changed

- remove legacy design and architecture generation scripts

## [0.4.1](https://github.com/tierlabx/widget-rs/releases/tag/v0.4.1) - 2026-08-13

### Fixed

- lock
- 修复 BreakOverlay 监听全局状态时的严重内存泄漏

## [0.4.0](https://github.com/tierlabx/widget-rs/releases/tag/v0.4.0) - 2026-08-13

### Added

- 增加允许跳过和推迟的设置项
- add pre-commit git hook for automated code checks

### Fixed

- 彻底修复多屏幕下休息遮罩在边缘的几像素漏光缝隙
- 修复跨分辨率和缩放显示器切换时的窗口大小恢复与主屏幕识别
- 修复没设置置顶，还经常压不下去的问题

### Changed

- 重构底层应用架构并按功能域拆分目录结构

## [0.3.2](https://github.com/tierlabx/widget-rs/releases/tag/v0.3.2) - 2026-08-12

### Fixed

- 修复便签md预览模式下无法滚动的bug

## [0.3.1](https://github.com/tierlabx/widget-rs/releases/tag/v0.3.1) - 2026-08-12

### Fixed

- skip destination folder selection dialog on upgrade

## [0.3.0](https://github.com/tierlabx/widget-rs/releases/tag/v0.3.0) - 2026-08-12

### Added

- 增加自动更新功能

### Changed

- 重构控制面板布局

## [0.2.2](https://github.com/tierlabx/widget-rs/releases/tag/v0.2.2) - 2026-08-12

### Fixed

- use repeated -f flag for cargo-packager multi-format args

### Changed

- 窗口能力封装，消除样板代码

## [0.2.2](https://github.com/tierlabx/widget-rs/releases/tag/v0.2.2) - 2026-08-12

### Fixed

- use repeated -f flag for cargo-packager multi-format args

### Changed

- 窗口能力封装，消除样板代码

## [0.2.1](https://github.com/tierlabx/widget-rs/releases/tag/v0.2.1) - 2026-08-07

### Fixed

- 修复action 发布

## [0.2.0](https://github.com/tierlabx/widget-rs/releases/tag/v0.2.0) - 2026-08-07

### Added

- implement automated release management tools for version bumping, CHANGELOG generation, and Cargo.toml updates
- 修复窗口漂移
- add stretchly plugin with full-screen break overlay and native window hook support

### Fixed

- ci 发布工具
- 修复多屏，及内存优化
- 修复内存泄漏问题
- 修复stretchly 遮罩问题
- 修复便签主题

### Changed

- remove dist-workspace.toml and update contribution guidelines


## [0.1.0](https://github.com/tierlabx/widget-rs/releases/tag/v0.1.0) - 2026-07-31

### Added

- 小组件持久化
- 自动重新聚焦了一次,系统托盘切换显示隐藏
- 数据持久化

### Fixed

- 修复fmt错误
- 修复release
- 修复顶部缩放问题
- 修复鼠标穿透，等问题
- 修复开机启动功能bug
- 修复启动项问题
- 修改设置
- 小部件设备修改
- 修复位置像素偏移
- 完成排版时保存
- 优化logo
- 防止win+d最下化组件
- 增加打包脚本
- 修复warning
- 实现大部分功能
- 增加编辑模式
- 更新目录结构

### Other

- release v0.1.0
- 更新文档，规则，修复warning
- 增加注释
