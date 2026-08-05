use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// P3 — 今日休息统计（每日自动清零）
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StretchlyStats {
    /// 记录日期 yyyy-mm-dd，用于跨日重置
    pub date: String,
    /// 当日完成的微休次数
    pub mini_breaks_done: u32,
    /// 当日完成的长休次数
    pub long_breaks_done: u32,
    /// 当日跳过次数（手动点「结束休息」）
    pub breaks_skipped: u32,
    /// 当日推迟次数
    pub breaks_postponed: u32,
    /// 当日累计专注分钟数（整数）
    pub focus_minutes: u32,
}

impl StretchlyStats {
    /// 如果日期不是今天则自动清零
    pub fn ensure_today(&mut self) {
        let today = today_date_string();
        if self.date != today {
            *self = StretchlyStats {
                date: today,
                ..Default::default()
            };
        }
    }
}

fn today_date_string() -> String {
    // 使用系统时间格式化 yyyy-mm-dd（不引入 chrono）
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // UTC 秒数转日期（简单整数算法，UTC+8 偏移 8*3600）
    let secs = secs + 8 * 3600;
    let days = secs / 86400;
    // 从 1970-01-01 推算年月日（Zeller / 儒略日算法）
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakState {
    Working,
    MiniBreak,
    LongBreak,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StretchlyConfig {
    /// 微休间隔（秒）
    pub mini_break_interval: u64,
    /// 微休时长（秒）
    pub mini_break_duration: u64,
    /// 长休间隔（秒）
    pub long_break_interval: u64,
    /// 长休时长（秒）
    pub long_break_duration: u64,
    /// 休息前预警时间（秒）
    pub warning_seconds: u64,
    /// 休息开始后多少秒才允许跳过
    pub skip_delay_seconds: u64,
    /// 推迟的分钟数
    pub postpone_minutes: u64,
}

impl Default for StretchlyConfig {
    fn default() -> Self {
        Self {
            mini_break_interval: 10 * 60,
            mini_break_duration: 20,
            long_break_interval: 30 * 60,
            long_break_duration: 5 * 60,
            warning_seconds: 30,
            skip_delay_seconds: 5,
            postpone_minutes: 5,
        }
    }
}

pub struct StretchlyModel {
    pub state: BreakState,
    pub config: StretchlyConfig,
    pub current_state_start: Instant,
    pub mini_breaks_taken: u32,
    /// 当前是否处于暂停状态（仅 Working 时有效）
    pub is_paused: bool,
    /// 累积暂停时长（计算 elapsed 时从中减去）
    pause_offset: Duration,
    /// 等待在下次状态切换时生效的新配置
    pending_config: Option<StretchlyConfig>,
    /// P3 — 今日统计
    pub stats: StretchlyStats,
    /// P3 — 专注分钟累计用的秒计数器
    focus_seconds_acc: u32,
    /// P3 — 上次 tick 时是否处于休息中（用于检测休息完成）
    prev_on_break: bool,
}

impl Default for StretchlyModel {
    fn default() -> Self {
        let mut stats = StretchlyStats::default();
        stats.ensure_today();
        Self {
            state: BreakState::Working,
            config: StretchlyConfig::default(),
            current_state_start: Instant::now(),
            mini_breaks_taken: 0,
            is_paused: false,
            pause_offset: Duration::ZERO,
            pending_config: None,
            stats,
            focus_seconds_acc: 0,
            prev_on_break: false,
        }
    }
}

impl StretchlyModel {
    pub fn new(config: Option<StretchlyConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            ..Default::default()
        }
    }

    /// 排队一个配置更新，在下次状态切换时生效，不打断当前阶段
    pub fn queue_config_update(&mut self, new_config: StretchlyConfig) {
        self.pending_config = Some(new_config);
    }

    /// 有效经过时间（已减去暂停偏移）
    fn elapsed(&self) -> Duration {
        self.current_state_start
            .elapsed()
            .saturating_sub(self.pause_offset)
    }

    /// 当前状态的总时长
    pub fn total_duration(&self) -> Duration {
        match self.state {
            BreakState::Working => Duration::from_secs(self.config.mini_break_interval),
            BreakState::MiniBreak => Duration::from_secs(self.config.mini_break_duration),
            BreakState::LongBreak => Duration::from_secs(self.config.long_break_duration),
        }
    }

    /// 当前状态剩余时间
    pub fn time_remaining(&self) -> Duration {
        self.total_duration().saturating_sub(self.elapsed())
    }

    /// 当前状态进度（0.0 = 刚开始，1.0 = 完成）
    pub fn progress(&self) -> f32 {
        let total = self.total_duration();
        if total.is_zero() {
            return 1.0;
        }
        (self.elapsed().as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
    }

    /// 是否处于休息前的预警窗口
    pub fn is_warning(&self) -> bool {
        self.state == BreakState::Working && !self.is_paused && {
            let rem = self.time_remaining();
            rem <= Duration::from_secs(self.config.warning_seconds) && !rem.is_zero()
        }
    }

    /// 是否正在休息
    pub fn is_on_break(&self) -> bool {
        matches!(self.state, BreakState::MiniBreak | BreakState::LongBreak)
    }

    /// 每秒 tick：处理暂停偏移和状态转换。返回 true 表示发生了状态切换。
    pub fn tick(&mut self) -> bool {
        // 跨日自动清零统计
        self.stats.ensure_today();

        let on_break = self.is_on_break();

        // 专注分钟累计（仅 Working 且非暂停）
        if !on_break && !self.is_paused {
            self.focus_seconds_acc += 1;
            if self.focus_seconds_acc >= 60 {
                self.focus_seconds_acc = 0;
                self.stats.focus_minutes += 1;
            }
        }

        // 检测休息自然完成（上次在休息中，这次 time_remaining 归零自动转换）
        if self.is_paused {
            self.pause_offset += Duration::from_secs(1);
            self.prev_on_break = on_break;
            return false;
        }
        if self.time_remaining().is_zero() {
            // 休息自然结束（非跳过）→ 记录完成次数
            if on_break {
                match self.state {
                    BreakState::MiniBreak => self.stats.mini_breaks_done += 1,
                    BreakState::LongBreak => self.stats.long_breaks_done += 1,
                    BreakState::Working => {}
                }
            }
            self.prev_on_break = on_break;
            self.transition_next_state();
            return true;
        }
        self.prev_on_break = on_break;
        false
    }

    /// 立即跳过当前阶段（手动点「结束休息」）
    pub fn skip(&mut self) {
        // 手动跳过休息 → 记录跳过次数
        if self.is_on_break() {
            self.stats.breaks_skipped += 1;
        }
        self.transition_next_state();
    }

    /// 推迟即将到来的休息（仅在 Working / Warning 状态下有效）
    pub fn postpone(&mut self) {
        if self.state == BreakState::Working {
            self.stats.breaks_postponed += 1;
            let extra = Duration::from_secs(self.config.postpone_minutes * 60);
            self.pause_offset += extra;
        }
    }

    /// 切换暂停/继续（仅在 Working 状态下有效）
    pub fn toggle_pause(&mut self) {
        if self.state == BreakState::Working {
            self.is_paused = !self.is_paused;
        }
    }

    /// 立即应用新配置并将计时器重置为 Working 状态从头开始
    pub fn apply_config_now(&mut self, new_config: StretchlyConfig) {
        self.config = new_config;
        self.pending_config = None;
        self.state = BreakState::Working;
        self.current_state_start = Instant::now();
        self.pause_offset = Duration::ZERO;
        self.is_paused = false;
        self.mini_breaks_taken = 0;
    }

    /// 每个长休周期内包含多少次微休
    pub fn mini_breaks_in_cycle(&self) -> u32 {
        (self.config.long_break_interval / self.config.mini_break_interval).max(1) as u32
    }

    /// 跳过当前休息，并在新的工作阶段追加 postpone_minutes 的延迟
    pub fn skip_and_postpone(&mut self) {
        if self.is_on_break() {
            self.stats.breaks_skipped += 1;
            self.stats.breaks_postponed += 1;
        }
        self.transition_next_state(); // 回到 Working 状态
        let extra = Duration::from_secs(self.config.postpone_minutes * 60);
        self.pause_offset += extra;
    }

    fn transition_next_state(&mut self) {
        // 在状态切换点应用排队的配置
        if let Some(cfg) = self.pending_config.take() {
            self.config = cfg;
        }
        match self.state {
            BreakState::Working => {
                self.mini_breaks_taken += 1;
                let intervals =
                    (self.config.long_break_interval / self.config.mini_break_interval).max(1);
                if self.mini_breaks_taken >= intervals as u32 {
                    self.state = BreakState::LongBreak;
                    self.mini_breaks_taken = 0;
                } else {
                    self.state = BreakState::MiniBreak;
                }
            }
            BreakState::MiniBreak | BreakState::LongBreak => {
                self.state = BreakState::Working;
            }
        }
        self.current_state_start = Instant::now();
        self.pause_offset = Duration::ZERO;
        self.is_paused = false;
    }
}
