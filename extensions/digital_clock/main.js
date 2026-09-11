/**
 * 极简数字时钟 (GPUI Shell JavaScript 扩展插件)
 * 
 * 本脚本直接被 GPUI 宿主环境的 JS 引擎执行。
 * 宿主每次渲染周期会调用 `render(context)` 获取声明式节点树，
 * 并以原生 GPUI 元素无损实时渲染，零 DOM、零 WebView 损耗。
 */
function render(context) {
    var now = new Date(context.timestamp || Date.now());
    var hours = String(now.getHours()).padStart(2, '0');
    var minutes = String(now.getMinutes()).padStart(2, '0');
    var seconds = String(now.getSeconds()).padStart(2, '0');
    var timeStr = hours + ":" + minutes + ":" + seconds;

    var year = now.getFullYear();
    var month = String(now.getMonth() + 1).padStart(2, '0');
    var day = String(now.getDate()).padStart(2, '0');
    var weekdays = ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"];
    var weekStr = weekdays[now.getDay()];
    var dateStr = year + "-" + month + "-" + day + " " + weekStr;

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
                text: timeStr,
                style: {
                    font_size: "32px",
                    font_weight: "bold",
                    color: "#60a5fa"
                }
            },
            {
                type: "text",
                text: dateStr,
                style: {
                    font_size: "12px",
                    font_weight: "normal",
                    color: "#94a3b8"
                }
            }
        ]
    };
}
