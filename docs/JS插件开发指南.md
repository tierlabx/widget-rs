# Widget-RS JavaScript 插件开发指南 (基于 GPUI Shell 架构)

本文档面向希望使用 JavaScript 编写桌面小组件的开发者。`widget-rs` 集成了 `gpui-kit` 生态官方的 **GPUI Shell** 扩展系统，允许开发者在无需搭建 Rust 编译环境的情况下，通过轻量级 JavaScript 脚本（ES Modules + 原生 QuickJS JIT 执行环境）声明式构建桌面小组件。

---

## 1. 架构理念与特性

- **原生渲染，零 DOM 开销**：区别于传统 Electron 或 WebView 方案，GPUI Shell 的 JavaScript 引擎直接运行在高性能嵌入式 QuickJS 虚拟机中，脚本通过声明式 `View` 类与构建链，直接映射为 GPUI 顶层 GPU 原生元素。
- **完整的 JavaScript 执行能力**：支持 ES Module、类面向对象继承、计算属性、循环迭代、条件分支、事件处理 (`on_click`) 以及原生定时器 (`cx.timer.every`)。
- **继承核心窗口能力**：自动享受 `widget-rs` 的 Win32 穿透、`Win+D` 桌面绑定常驻、毛玻璃透明背景、置顶层级控制以及拖拽移动能力。

---

## 2. 插件目录结构

一个小组件扩展为一个独立的文件夹，放置在扩展根目录下。标准目录结构如下：

```text
extensions/
└── my_widget/
    ├── widget.json       # 插件元数据与窗口配置（必需）
    ├── gpui-shell.json   # GPUI Shell 标准模块配置（可选）
    ├── main.js           # 业务逻辑与 View 渲染入口（必需）
    └── icon.png          # 插件图标（可选，支持 .png 或 .svg）
```

---

## 3. 配置文件

### 1. `widget.json` (小组件窗口属性)

用于定义小组件在 `widget-rs` 控制面板中的呈现方式与窗口样式：

```json
{
  "id": "my_custom_widget",
  "name": "我的自定义小组件",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "基于 GPUI Shell 架构的桌面小组件",
  "main": "main.js",
  "icon": "clock",
  "window": {
    "width": 310.0,
    "height": 92.0,
    "transparent": true,
    "blurred": true
  }
}
```

### 2. `gpui-shell.json` (GPUI Shell 标准清单)

定义给底层 `gpui-shell` 引擎识别的模块信息：

```json
{
  "id": "my_custom_widget",
  "name": "我的自定义小组件",
  "version": "1.0.0",
  "shell-version": "0.6.1",
  "entry": "main.js"
}
```

---

## 4. 脚本入口 `main.js`

扩展通过默认导出一个继承自 `gpui-kit` 的 `View` 类来构建界面。

### 核心生命周期与 API

- **`init(props, cx)`**：组件初始化生命周期，用于初始化内部状态、注册定时器或异步数据监听。
- **`render(cx)`**：渲染函数，返回通过 `h_flex()`、`v_flex()`、`div()` 等构建的原生节点树。
- **`cx.notify()`**：当状态变更时触发此方法，通知 GPUI 进行高效差异重绘。
- **`cx.timer.every(interval_ms, callback)`**：注册周期定时器任务。

### 示例代码

```javascript
import { View, div } from "gpui-kit";
import { h_flex, v_flex } from "gpui-base";

export default class DigitalClock extends View {
  init(_props, cx) {
    this.hoursMinutes = "--:--";
    this.seconds = "00s";
    this.date = "--/--";
    this.updateTime();

    // 每秒更新时间状态并通知 GPUI 刷新
    this.timer = cx.timer.every(1000, () => {
      this.updateTime();
      cx.notify();
    });
  }

  updateTime() {
    const now = new Date();
    const pad = (n) => String(n).padStart(2, "0");
    this.hoursMinutes = `${pad(now.getHours())}:${pad(now.getMinutes())}`;
    this.seconds = `${pad(now.getSeconds())}s`;

    const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
    this.date = `${pad(now.getMonth() + 1)}/${pad(now.getDate())} ${weekdays[now.getDay()]}`;
  }

  render(_cx) {
    return h_flex()
      .w_full()
      .h_full()
      .px(16)
      .py(12)
      .gap(16)
      .items_center()
      .justify_between()
      .bg("#0f172a65")
      .rounded(16)
      .border(1)
      .border_color("#ffffff18")
      .child(
        div()
          .text_size(38)
          .font_bold()
          .text_color("#f8fafc")
          .child(this.hoursMinutes)
      )
      .child(
        v_flex()
          .gap(6)
          .justify_center()
          .child(
            h_flex()
              .px(6)
              .py(2)
              .rounded(6)
              .bg("#00d99222")
              .border(1)
              .border_color("#00d99244")
              .child(
                div()
                  .text_size(11)
                  .font_bold()
                  .text_color("#00d992")
                  .child(this.seconds)
              )
          )
          .child(
            div()
              .text_size(11)
              .text_color("#94a3b8")
              .child(this.date)
          )
      );
  }
}
```

---

## 5. 常用样式链式方法说明

GPUI Shell 提供了与原生 GPUI 完全对齐的链式样式调用方法：

### 布局与尺寸
- `.w_full()` / `.h_full()` / `.size_full()`：填满父级宽高
- `.flex_1()`：占用剩余弹性空间
- `.px(val)` / `.py(val)` / `.p(val)`：内边距
- `.gap(val)`：Flex 子项间距
- `.items_center()` / `.justify_center()` / `.justify_between()`：对齐方式

### 背景与边框
- `.bg(hex_or_token)`：背景颜色（支持带透明度的 HEX，如 `"#0f172a65"`）
- `.rounded(radius)`：圆角半径（像素）
- `.border(width)`：边框粗细
- `.border_color(hex_or_token)`：边框颜色

### 字体与排版
- `.text_size(size)`：字体大小（像素）
- `.font_bold()` / `.font_normal()` / `.font_weight(weight)`：字重设置
- `.text_color(color)`：文字颜色

---

## 6. 插件存放目录与调试

### 扩展目录位置
`widget-rs` 会自动扫描并监视以下目录：
1. **用户目录**：`%APPDATA%/tierlabx/widget-rs/extensions/`
2. **本地目录**：程序执行目录下的 `./extensions/`

### 在控制面板中调试与管理
1. 打开 `widget-rs` 控制面板，切换到 **小部件库**。
2. 点击顶部的分类标签 **JS 扩展**，即可筛选所有由 JavaScript 编写的外部扩展。
3. 点击右上角的 **打开扩展目录** 按钮，系统资源管理器将直接弹出扩展文件夹。
4. 修改你的 `main.js` 后，点击右上角的 **刷新扩展** 按钮即可一键热重新载入，无需重启主程序。
