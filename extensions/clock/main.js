import { View, div, image } from "gpui-kit";
import { h_flex, v_flex } from "gpui-base";
import { read_config } from "widget-rs";

/**
 * 根据和风天气图标码或文字描述匹配对应的 SVG 矢量图标文件路径
 * @param {string} iconCode
 * @param {string} text
 * @returns {string}
 */
function getWeatherIconPath(iconCode, text = "") {
  const code = String(iconCode || "").trim();
  const desc = String(text || "").trim();

  // 优先匹配复合天气（多云/少云/晴间多云）
  if (["101", "102", "103", "151", "152", "153"].includes(code) || desc.includes("多云") || desc.includes("少云")) return "icons/partly-cloudy.png";
  if (code === "100" || code === "150" || desc.includes("晴")) return "icons/sunny.png";
  if (code === "104" || desc.includes("阴")) return "icons/cloudy.png";
  if (["302", "303", "304"].includes(code) || desc.includes("雷")) return "icons/thunder.png";
  if ((code.startsWith("3") && code.length === 3) || desc.includes("雨")) return "icons/rainy.png";
  if ((code.startsWith("4") && code.length === 3) || desc.includes("雪") || desc.includes("雹")) return "icons/snowy.png";
  if ((code.startsWith("5") && code.length === 3) || /雾|霾|风|沙|尘/.test(desc)) return "icons/fog.png";
  return "icons/partly-cloudy.png";
}

/**
 * 安全读取和风天气配置：
 * 1. 优先读取插件目录下的私有 `config.json`（已被 .gitignore 忽略，保护个人密钥不会被提交到 Git）
 * 2. 其次回退读取 `manifest.json` 中的 `settings` 字段
 * 3. 支持热读取：用户修改 config.json 后点击天气胶囊即可实时生效，无需重启应用
 */
function loadWeatherConfig() {
  const defaults = {
    apiHost: "devapi.qweather.com",
    apiKey: "",
    location: "auto",
    updateIntervalMs: 20 * 60 * 1000,
  };

  try {
    const raw = read_config("clock");
    if (raw && raw.trim() !== "" && raw !== "{}") {
      /** @type {any} */
      const userConfig = JSON.parse(raw);
      return { ...defaults, ...userConfig };
    }
  } catch (err) {
    console.error("[时钟插件] 读取外部配置失败:", err);
  }
  return defaults;
}

/**
 * 时钟 (GPUI Shell JavaScript 扩展小组件)
 * 
 * 基于官方 gpui-shell 架构运行，拥有完整的原生 JavaScript 运行环境：
 * - 支持 ES Module 标准语法与面向对象 View 类
 * - 原生 fetch 网络请求调用和风天气实时 API
 * - 纯原生状态响应式更新 (通过 cx.notify 触发渲染)
 * - 与底层 Win32 DirectComposition 亚克力磨砂透明壁纸直通
 */
export default class Clock extends View {
  init(_props, cx) {
    this.config = loadWeatherConfig();
    this.hours = "--";
    this.minutes = "--";
    this.seconds = "00s";
    this.ampm = "AM";
    this.colonVisible = true;
    this.weekday = "周--";
    this.monthDay = "--/--";
    this.weather = this.config.apiKey ? "定位天气中..." : "和风未填Key";
    this.weatherIcon = "";
    this.tempRange = "";
    this.forecast = [];
    this.isRefreshing = false;

    // 立即执行初次时间计算
    this.updateTime();

    // 注册时钟定时任务：每秒更新时间并通知 GPUI 刷新
    this.timer = cx.timer.every(1000, () => {
      this.updateTime();
      cx.notify();
    });

    // 初次获取和风天气与预报（含自动定位）
    cx.spawn(async (taskCx) => {
      await this.updateWeather();
      taskCx.notify();
    });

    // 注册天气定时刷新任务（默认 20 分钟）
    this.weatherTimer = cx.timer.every(this.config.updateIntervalMs || 1200000, (timerCx) => {
      timerCx.spawn(async (taskCx) => {
        await this.updateWeather();
        taskCx.notify();
      });
    });
  }

  updateTime() {
    const now = new Date();
    const pad = (n) => String(n).padStart(2, "0");
    const h = now.getHours();
    this.hours = pad(h);
    this.minutes = pad(now.getMinutes());
    this.seconds = `${pad(now.getSeconds())}s`;
    this.ampm = h >= 12 ? "PM" : "AM";
    this.colonVisible = now.getSeconds() % 2 === 0;

    const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
    this.weekday = weekdays[now.getDay()];
    this.monthDay = `${pad(now.getMonth() + 1)}/${pad(now.getDate())}`;
  }

  /**
   * 自动通过 IP 地址定位当前所在城市的经纬度及城市名
   */
  async getAutoLocation() {
    const endpoints = [
      {
        url: "https://ipwho.is/",
        parse: (d) => d.success && d.latitude && { coords: `${Number(d.longitude).toFixed(2)},${Number(d.latitude).toFixed(2)}`, city: d.city || "" },
      },
      {
        url: "https://api.ip.sb/geoip",
        parse: (d) => d.longitude && { coords: `${Number(d.longitude).toFixed(2)},${Number(d.latitude).toFixed(2)}`, city: d.city || "" },
      },
      {
        url: "http://ip-api.com/json/?lang=zh-CN",
        parse: (d) => d.status === "success" && { coords: `${Number(d.lon).toFixed(2)},${Number(d.lat).toFixed(2)}`, city: d.city || d.regionName || "" },
      },
    ];

    for (const { url, parse } of endpoints) {
      try {
        const res = await fetch(url);
        if (res.ok) {
          /** @type {any} */
          const d = await res.json();
          const loc = parse(d);
          if (loc && loc.coords) return loc;
        }
      } catch (_e) {
        // 继续尝试下一个备选定位源
      }
    }
    return null;
  }

  /**
   * 异步调用和风天气 API 并行获取实时天气状况与 3 天天气预报
   */
  async updateWeather() {
    this.config = loadWeatherConfig();
    const { apiKey, apiHost, location } = this.config;

    if (!apiKey || apiKey.trim() === "" || apiKey === "YOUR_QWEATHER_KEY") {
      this.weather = "和风未填Key";
      return;
    }

    try {
      let queryLocation = (location || "auto").trim();
      let cityName = "";

      // 自动 IP 定位模式
      if (queryLocation === "auto" || queryLocation === "") {
        const autoLoc = await this.getAutoLocation();
        if (autoLoc) {
          queryLocation = autoLoc.coords;
          cityName = autoLoc.city;
        } else {
          queryLocation = "116.40,39.90";
          cityName = "北京";
        }
      }

      const host = (apiHost || "devapi.qweather.com")
        .trim()
        .replace(/^https?:\/\//, "")
        .replace(/\/$/, "");

      const nowUrl = `https://${host}/v7/weather/now?location=${encodeURIComponent(queryLocation)}&key=${apiKey.trim()}&lang=zh`;
      const forecastUrl = `https://${host}/v7/weather/3d?location=${encodeURIComponent(queryLocation)}&key=${apiKey.trim()}&lang=zh`;

      const [resNow, resForecast] = await Promise.all([
        fetch(nowUrl).catch(() => null),
        fetch(forecastUrl).catch(() => null),
      ]);

      if (resNow && resNow.ok) {
        /** @type {any} */
        const data = await resNow.json();
        if (data.code === "200" && data.now) {
          const cityPrefix = cityName ? `${cityName} ` : "";
          this.weather = `${cityPrefix}${data.now.text} ${data.now.temp}°C`;
          this.weatherIcon = getWeatherIconPath(data.now.icon, data.now.text);
        } else {
          console.error("[时钟插件] 和风天气响应异常:", data.code);
          this.weather = `和风Err ${data.code}`;
          this.weatherIcon = "";
        }
      }

      if (resForecast && resForecast.ok) {
        /** @type {any} */
        const fData = await resForecast.json();
        if (fData.code === "200" && Array.isArray(fData.daily) && fData.daily.length > 0) {
          const labels = ["今天", "明天", "后天"];
          this.forecast = fData.daily.slice(0, 3).map((item, idx) => ({
            label: labels[idx] || item.fxDate.slice(5),
            text: item.textDay,
            tempMin: `${item.tempMin}°`,
            tempMax: `${item.tempMax}°`,
            icon: getWeatherIconPath(item.iconDay, item.textDay),
          }));
          if (fData.daily[0]) {
            this.tempRange = `${fData.daily[0].tempMin}°~${fData.daily[0].tempMax}°`;
          }
        }
      }
    } catch (err) {
      console.error("[时钟插件] 请求和风天气失败:", err);
      const errMsg = err instanceof Error ? err.message : String(err || "获取失败");
      this.weather = errMsg.length > 12 ? errMsg.slice(0, 12) : errMsg;
      this.weatherIcon = "";
    } finally {
      this.isRefreshing = false;
    }
  }

  render(_cx) {
    return h_flex()
      .w_full().h_full().px(16).py(10).gap(12)
      .items_center().justify_between()
      .bg("#080f1ecc").rounded(16).border(1).border_color("#ffffff1a")
      .shadow_lg().hover((el) => el.border_color("#ffffff2e"))
      .child(
        // 左侧：时钟核心区
        v_flex().gap(4).justify_center().child(
          // 大时间行
          h_flex().items_center()
            .child(div().text_size(36).font_bold().text_color("#f8fafc").child(this.hours))
            .child(div().text_size(28).font_bold().text_color("#94a3b8")
              .opacity(this.colonVisible ? 1.0 : 0.25).transition("opacity", 350).mx(1).child(":"))
            .child(div().text_size(36).font_bold().text_color("#f8fafc").child(this.minutes))
            .child(div().ml(4).mb(14).px(4).py(1).rounded(4).bg("#ffffff12")
              .border(1).border_color("#ffffff18").text_size(9).font_bold().text_color("#94a3b8").child(this.ampm))
        ).child(
          // 周历、日期与心跳秒针胶囊
          h_flex().gap(5).items_center()
            .child(div().px(4).py(1).rounded(4).bg("#ffffff0f").border(1).border_color("#ffffff15")
              .text_size(10).font_bold().text_color("#cbd5e1").child(this.weekday))
            .child(div().text_size(10).text_color("#94a3b8").child(this.monthDay))
            .child(
              h_flex().px(4).py(1).gap(3).rounded(4).bg("#00d9921c").border(1).border_color("#00d99244").items_center()
                .child(div().size(4).rounded(99).bg("#00d992").opacity(this.colonVisible ? 1.0 : 0.35).transition("opacity", 300))
                .child(div().text_size(10).font_bold().text_color("#00d992").child(this.seconds))
            )
        )
      )
      .child(
        // 中间微光分隔线
        div().w(1).h(66).bg("#ffffff14").rounded(1)
      )
      .child(
        // 右侧：实时气象与未来 3 天天气预报看板
        (() => {
          const weatherPanel = v_flex().gap(5).justify_center().flex_1().cursor_pointer()
            .on_click((_, clickCx) => {
              this.isRefreshing = true;
              this.weather = "刷新中...";
              clickCx.notify();
              clickCx.spawn(async (taskCx) => {
                await this.updateWeather();
                taskCx.notify();
              });
            });

          // 上层：实时天气条
          const liveRow = h_flex().w_full().items_center().justify_between().px(6).py(2)
            .rounded(6).bg("#38bdf814").border(1).border_color("#38bdf833")
            .hover((el) => el.bg("#38bdf824").border_color("#38bdf866"))
            .active((el) => el.bg("#38bdf838"));

          const liveLeft = h_flex().gap(5).items_center();
          if (this.weatherIcon) {
            liveLeft.child(image(this.weatherIcon).size(15).flex_shrink_0());
          }
          liveLeft.child(div().text_size(11).font_bold().text_color("#38bdf8")
            .opacity(this.isRefreshing ? 0.5 : 1.0).transition("opacity", 250).child(this.weather));
          liveRow.child(liveLeft);

          if (this.tempRange) {
            liveRow.child(div().px(4).py(1).rounded(4).bg("#38bdf818")
              .text_size(9).font_bold().text_color("#7dd3fc").child(this.tempRange));
          }
          weatherPanel.child(liveRow);

          // 下层：未来 3 天天气预报微卡片
          const forecastRow = h_flex().w_full().gap(4).items_center().justify_between();
          const items = this.forecast.length > 0 ? this.forecast : [
            { label: "今天", tempMin: "--", tempMax: "--", icon: "icons/sunny.png" },
            { label: "明天", tempMin: "--", tempMax: "--", icon: "icons/partly-cloudy.png" },
            { label: "后天", tempMin: "--", tempMax: "--", icon: "icons/cloudy.png" },
          ];

          for (const item of items) {
            forecastRow.child(
              h_flex().flex_1().px(4).py(2).gap(2).rounded(5).bg("#ffffff08").border(1).border_color("#ffffff0f")
                .items_center().justify_between()
                .hover((el) => el.bg("#ffffff14").border_color("#ffffff20"))
                .child(div().text_size(9).text_color("#94a3b8").child(item.label))
                .child(image(item.icon).size(12).flex_shrink_0())
                .child(div().text_size(9).font_bold().text_color("#e2e8f0").child(`${item.tempMin}/${item.tempMax}`))
            );
          }
          weatherPanel.child(forecastRow);

          return weatherPanel;
        })()
      );
  }
}
