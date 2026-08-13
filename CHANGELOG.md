# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
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
