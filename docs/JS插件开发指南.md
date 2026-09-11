# Widget-RS JavaScript 插件开发指南 (GPUI Shell)

本文档面向希望使用 JavaScript 编写动态桌面小组件的开发者。`widget-rs` 集成了 **GPUI Shell** 扩展系统，允许开发者在无需搭建 Rust 编译环境的情况下，通过轻量级 JavaScript 脚本声明式构建桌面小组件。

---

## 1. 架构理念与特性

- **原生渲染，零 DOM 开销**：区别于传统 Electron 或 WebView 方案，GPUI Shell 的 JavaScript 引擎直接运行在轻量级嵌入式解释器中，脚本输出的节点树会直接映射为 GPUI 顶层 GPU 原生元素。
- **免重新编译**：放置在扩展目录即可热加载、实时生效，普通开发者或社区用户无需安装 Rust 工具链。
- **继承核心窗口能力**：自动享受 `widget-rs` 的 Win32 穿透、`Win+D` 桌面绑定常驻、毛玻璃透明背景、置顶层级控制以及拖拽移动能力。

---

## 2. 插件目录结构

一个小组件扩展为一个独立的文件夹，放置在扩展根目录下。标准目录结构如下：

```text
extensions/
└── my_widget/
    ├── widget.json       # 插件清单与窗口配置（必需）
    ├── main.js           # 逻辑脚本与 UI 渲染入口（必需）
    └── icon.png          # 插件图标（可选，支持 .png 或 .svg）
```

---

## 3. 插件清单 `widget.json`

每个扩展必须在根目录下包含 `widget.json`（或兼容 `gpui-shell.json`），用于定义插件元数据与初始窗口属性：

```json
{
  "id": "my_custom_widget",
  "name": "我的自定义小组件",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "基于 JavaScript 扩展构建的桌面小组件",
  "main": "main.js",
  "icon": "clock",
  "window": {
    "width": 280.0,
    "height": 120.0,
    "transparent": true,
    "blur": true
  }
}
```

### 字段说明

| 字段 | 类型 | 说明 |
| :--- | :--- | :--- |
| `id` | `string` | 插件全局唯一英文标识符（必填，只允许小写字母、数字与下划线） |
| `name` | `string` | 在控制面板和小部件库中展示的名称（必填） |
| `version` | `string` | 语义化版本号，如 `1.0.0` |
| `author` | `string` | 作者名称 |
| `description` | `string` | 插件功能简述 |
| `main` | `string` | 脚本入口文件名，默认 `main.js` |
| `icon` | `string` | 图标名称或图标相对文件路径 |
| `window.width` | `number` | 窗口默认启动逻辑宽度（像素） |
| `window.height` | `number` | 窗口默认启动逻辑高度（像素） |
| `window.transparent` | `boolean` | 是否开启全透明直通背景（默认 `true`） |
| `window.blur` | `boolean` | 是否开启 Windows DWM 系统毛玻璃亚克力效果（默认 `true`） |

---

## 4. 脚本入口 `main.js`

扩展通过导出一个全局函数 `render(context)` 来构建界面。宿主引擎在每次定时更新周期调用该函数，并将其返回的节点树渲染为 GPUI 界面。

### 示例代码

```javascript
/**
 * 渲染函数
 * @param {Object} context 宿主注入的上下文信息
 * @param {number} context.timestamp 当前系统时间戳 (毫秒)
 * @param {string} context.formatted_time 格式化时间文本 (HH:MM:SS)
 * @returns {Object} 声明式节点树
 */
function render(context) {
    var now = new Date(context.timestamp || Date.now());
    var hours = String(now.getHours()).padStart(2, '0');
    var minutes = String(now.getMinutes()).padStart(2, '0');
    var seconds = String(now.getSeconds()).padStart(2, '0');
    var timeText = hours + ":" + minutes + ":" + seconds;

    return {
        type: "div",
        style: {
            display: "flex",
            flex_direction: "col",
            justify_content: "center",
            align_items: "center",
            width: "full",
            height: "full",
            padding: "12px",
            background_color: "#18181bE6",
            border_radius: "12px",
            border_color: "#3f3f46",
            border_width: "1px"
        },
        children: [
            {
                type: "text",
                text: timeText,
                style: {
                    font_size: "28px",
                    font_weight: "bold",
                    color: "#38bdf8"
                }
            },
            {
                type: "text",
                text: "GPUI Shell 驱动",
                style: {
                    font_size: "12px",
                    font_weight: "normal",
                    color: "#94a3b8"
                }
            }
        ]
    };
}
```

---

## 5. 支持的样式属性（Style Reference）

节点树支持以下常见的现代 UI 布局与样式属性：

### 容器与布局
- `display`: `"flex"`
- `flex_direction`: `"row"` | `"col"`
- `justify_content`: `"start"` | `"center"` | `"end"` | `"space_between"`
- `align_items`: `"start"` | `"center"` | `"end"`
- `gap`: 间距数值（如 `"8px"` 或数值 `8`）
- `padding`: 内边距数值（如 `"12px"` 或数值 `12`）
- `width` / `height`: `"full"` 或具体像素值（如 `"240px"`）

### 背景与边框
- `background_color`: 支持 HEX 颜色（如 `"#1e293b"`、`"#18181bE6"` 包含透明度）
- `border_radius`: 圆角大小（如 `"8px"` 或数值 `8`）
- `border_width`: 边框宽度（如 `"1px"` 或数值 `1`）
- `border_color`: 边框颜色（如 `"#475569"`）

### 文字排版
- `font_size`: 字体尺寸（如 `"14px"` 或数值 `14`）
- `font_weight`: `"normal"` | `"bold"`
- `color`: 文本颜色 HEX 编码（如 `"#f8fafc"`）

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
4. 修改你的 `main.js` 或 `widget.json` 后，点击右上角的 **刷新扩展** 按钮即可一键热重新载入，无需重启主程序。
