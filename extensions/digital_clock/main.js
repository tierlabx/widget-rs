import { View, div } from "gpui-kit";
import { h_flex, v_flex } from "gpui-base";

/**
 * 极简数字时钟 (GPUI Shell JavaScript 扩展小组件)
 * 
 * 基于官方 gpui-shell 架构运行，拥有完整的原生 JavaScript 运行环境：
 * - 支持 ES Module 标准语法与面向对象 View 类
 * - 纯原生状态响应式更新 (通过 cx.notify 触发渲染)
 * - 与底层 Win32 DirectComposition 亚克力磨砂透明壁纸直通
 */
export default class DigitalClock extends View {
  init(_props, cx) {
    this.hoursMinutes = "--:--";
    this.seconds = "00s";
    this.date = "--/--";
    this.weather = "晴 24°C";

    // 立即执行初次时间计算
    this.updateTime();

    // 注册定时任务：每秒更新时间并通知 GPUI 刷新
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
        // 左侧大字号高亮时分展示
        div()
          .text_size(38)
          .font_bold()
          .text_color("#f8fafc")
          .child(this.hoursMinutes)
      )
      .child(
        // 右侧多功能胶囊芯片与日期
        v_flex()
          .gap(6)
          .justify_center()
          .child(
            h_flex()
              .gap(6)
              .items_center()
              .child(
                // 动态秒针跳动胶囊
                h_flex()
                  .px(6)
                  .py(2)
                  .rounded(6)
                  .bg("#00d99222")
                  .border(1)
                  .border_color("#00d99244")
                  .items_center()
                  .justify_center()
                  .child(
                    div()
                      .text_size(11)
                      .font_bold()
                      .text_color("#00d992")
                      .child(this.seconds)
                  )
              )
              .child(
                // 天气信息芯片
                h_flex()
                  .px(6)
                  .py(2)
                  .rounded(6)
                  .bg("#38bdf822")
                  .border(1)
                  .border_color("#38bdf844")
                  .items_center()
                  .justify_center()
                  .child(
                    div()
                      .text_size(11)
                      .font_bold()
                      .text_color("#38bdf8")
                      .child(this.weather)
                  )
              )
          )
          .child(
            // 日期与星期显示
            div()
              .text_size(11)
              .text_color("#94a3b8")
              .child(this.date)
          )
      );
  }
}
