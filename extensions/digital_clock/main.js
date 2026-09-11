/**
 * 极简数字时钟 (GPUI Shell JavaScript 扩展插件)
 * 
 * 采用现代磨砂毛玻璃双栏胶囊设计：
 * - 背景：与 Todo/Fences 完全一致的 DirectComposition 原生亚克力半透明透视壁纸
 * - 左侧：大号极简高亮纯白时分显示
 * - 右侧：动态信号绿秒针胶囊芯片 + 精致日期星期副文本
 */
function render(context) {
    return /*WIDGET_UI_START*/{
        "type": "h_flex",
        "style": {
            "w_full": true,
            "h_full": true,
            "px": 16,
            "py": 12,
            "gap": 16,
            "items_center": true,
            "justify_between": true,
            "bg": "#0f172a65",
            "rounded": 16,
            "border_color": "#ffffff18",
            "border_width": 1
        },
        "children": [
            {
                "type": "text",
                "text": "{{hours_minutes}}",
                "style": {
                    "font_size": 40,
                    "bold": true,
                    "color": "#f8fafc"
                }
            },
            {
                "type": "v_flex",
                "style": {
                    "gap": 6,
                    "items_center": false,
                    "justify_center": true
                },
                "children": [
                    {
                        "type": "h_flex",
                        "style": {
                            "px": 8,
                            "py": 2,
                            "rounded": 6,
                            "bg": "#00d99222",
                            "border_color": "#00d99244",
                            "border_width": 1,
                            "items_center": true,
                            "justify_center": true
                        },
                        "children": [
                            {
                                "type": "text",
                                "text": "{{seconds}}s",
                                "style": {
                                    "font_size": 11,
                                    "bold": true,
                                    "color": "#00d992"
                                }
                            }
                        ]
                    },
                    {
                        "type": "text",
                        "text": "{{date}}",
                        "style": {
                            "font_size": 11,
                            "color": "#94a3b8"
                        }
                    }
                ]
            }
        ]
    }/*WIDGET_UI_END*/;
}
