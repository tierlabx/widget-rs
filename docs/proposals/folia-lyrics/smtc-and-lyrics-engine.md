# Folia 歌词：SMTC 媒体监听与歌词引擎设计

本文档规划小部件如何监听 Windows 全局媒体播放器（SMTC），以及逐字歌词解析与高精度时间轴对齐机制。

---

## 1. Windows SMTC 监听协议架构

Windows 10/11 提供了原生的 `GlobalSystemMediaTransportControlsSessionManager` (WinRT API)，几乎所有主流播放器均已实现 SMTC 接入（网易云音乐、QQ 音乐、Spotify、Apple Music、Foobar2000、Chrome/Edge 网页媒体等）。

```
+--------------------------------------------------------------+
| 系统中运行的音乐播放器 (Netease, QQMusic, Spotify, Browser...)  |
+--------------------------------------------------------------+
                               ↓ (OS 级媒体管道广播)
+--------------------------------------------------------------+
| Windows SMTC Session Manager (WinRT)                         |
+--------------------------------------------------------------+
                               ↓ (跨线程事件订阅与异步信道)
+--------------------------------------------------------------+
| SmtcListener (widget-rs folia-lyrics 后台监听服务)            |
| - 当前会话绑定: SessionChanged                                |
| - 元数据获取: MediaProperties (Title, Artist, Album, Thumbnail)|
| - 播放控制态: PlaybackInfo (Playing, Paused, Timeline)       |
+--------------------------------------------------------------+
                               ↓ (标准内部媒体事件统一模型)
+--------------------------------------------------------------+
| 歌词匹配与时间对齐状态机 (LyricsTimelineEngine)                |
+--------------------------------------------------------------+
```

---

## 2. 状态监听与进度高精度外推算法

### 2.1 为什么需要外推算法 (Timeline Extrapolation)？
Windows SMTC 报告的播放进度通常不是每帧推送的，而是每秒触发 1 次或仅在暂停/拖动时触发。如果直接使用 SMTC 上报的静态时间，歌词动效会每秒“卡跳”一次。

### 2.2 本地高精度外推公式
设：
- $T_{base}$ 为 SMTC 上次同步时的曲目播放位置。
- $t_{sync}$ 为 SMTC 收到该更新时的系统单调时钟戳（`std::time::Instant::now()`）。
- $S$ 为当前播放速率（通常为 1.0）。

在任意渲染时刻 $t_{render}$：
$$\text{CurrentPosition} = T_{base} + (t_{render} - t_{sync}) \times S$$

当系统接收到 SMTC 的 `TimelinePropertiesChanged` 或 `PlaybackInfoChanged` 时，重新校准 $T_{base}$ 与 $t_{sync}$。如果误差超过 500ms（表明用户在播放器中执行了 Seek 拖动），立即重置当前歌词活跃行索引，并触发平滑跳行吸附。

---

## 3. 歌词解析引擎规范

引擎设计为可扩展的模块化解析管道，支持纯行级 LRC 与微秒级逐字格式：

### 3.1 支持格式优先级
1. **YRC (Yamaha/Netease 逐字格式)**：
   - 语法形如：`[1234,4560](1234,200,0)沉(1434,300,0)浸...`
   - 支持逐字开始绝对时间、持续时间及音高属性。
2. **QRC (QQ 音乐逐字加密/解密格式)**：
   - 语法形如：`[1234,4560]沉(200)浸(300)...`
3. **LRC (标准行级时间戳)**：
   - 语法形如：`[00:12.34]沉浸在桌面里的每一个音符`
   - 对于普通 LRC，小部件通过字数与音素估算自动生成匀速平滑扫光过渡，避免单调卡顿。

### 3.2 统一内部歌词数据模型
```rust
/// 单个字符/词的精确时间片
#[derive(Clone, Debug, PartialEq)]
pub struct WordTiming {
    pub text: String,
    pub start_ms: u32,
    pub duration_ms: u32,
}

/// 单行歌词
#[derive(Clone, Debug, PartialEq)]
pub struct LyricLine {
    pub line_index: usize,
    pub start_ms: u32,
    pub duration_ms: u32,
    pub raw_text: String,
    pub words: Vec<WordTiming>,
    pub translation: Option<String>,
}

/// 完整歌词对象
#[derive(Clone, Debug, Default)]
pub struct TrackLyrics {
    pub track_title: String,
    pub artist: String,
    pub lines: Vec<LyricLine>,
    pub is_word_by_word: bool,
}
```

---

## 4. 智能歌词检索与缓存流水线

当 SMTC 切换曲目后，后台异步任务启动歌词获取流程：

1. **本地二级缓存检查**：
   - 计算曲目标签哈希 `hash(Title + " " + Artist)`。
   - 优先读取 `%APPDATA%/widget-rs/lyrics_cache/<hash>.json`。
2. **在线检索与降级链 (Fallback Chain)**：
   - **Step 1 (高精逐字源)**：向网易云/QQ 音乐开放歌词 API 发起检索（请求 YRC/QRC 逐字数据）。
   - **Step 2 (全开源公共库)**：如未命中逐字源，向 LrcLib / 网易云发起标准 LRC 检索。
   - **Step 3 (无歌词纯音乐模式)**：若均未检索到，UI 自动切换为“纯音乐流光律动模式”，仅展示动态封面、频谱与曲目信息。
3. **防抖与竞态取消**：
   - 用户连续快速切歌时，使用异步任务令牌（`CancellationToken`）立即中止前一首歌曲的网络请求，避免过时数据乱序覆盖。
