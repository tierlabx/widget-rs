#[cfg(test)]
mod tests {
    use crate::model::{BreakState, StretchlyConfig, StretchlyModel};

    #[test]
    fn test_stretchly_state_transitions() {
        let mut config = StretchlyConfig::default();
        config.mini_break_interval = 1200; // 20 min
        config.mini_break_duration = 20;
        config.long_break_interval = 2400; // 2 mini breaks per long break
        config.long_break_duration = 300;

        let mut model = StretchlyModel::new(Some(config));
        assert_eq!(model.state, BreakState::Working);
        assert!(!model.is_on_break());

        // 第一次跳过 working -> 进入第 1 次微休
        model.skip();
        assert_eq!(model.state, BreakState::MiniBreak);
        assert!(model.is_on_break());

        // 结束微休 -> 回到 working
        model.skip();
        assert_eq!(model.state, BreakState::Working);

        // 第二次跳过 working -> 达到周期阈值，进入长休
        model.skip();
        assert_eq!(model.state, BreakState::LongBreak);
        assert!(model.is_on_break());

        // 结束长休 -> 回到 working
        model.skip();
        assert_eq!(model.state, BreakState::Working);
        assert_eq!(model.mini_breaks_taken, 0);
    }

    #[test]
    fn test_stretchly_pause_and_postpone() {
        let mut model = StretchlyModel::new(None);
        assert!(!model.is_paused);

        model.toggle_pause();
        assert!(model.is_paused);

        model.toggle_pause();
        assert!(!model.is_paused);

        let initial_postponed = model.stats.breaks_postponed;
        model.postpone();
        assert_eq!(model.stats.breaks_postponed, initial_postponed + 1);
    }
}
