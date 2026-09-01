#[cfg(test)]
mod tests {
    use crate::monitor::{clamp_to_work_area, find_best_monitor, MonitorInfo, Rect};

    #[test]
    fn test_find_best_monitor() {
        let monitors = vec![
            MonitorInfo {
                rc_monitor: Rect {
                    left: 0,
                    top: 0,
                    right: 1920,
                    bottom: 1080,
                },
                rc_work: Rect {
                    left: 0,
                    top: 0,
                    right: 1920,
                    bottom: 1040,
                },
                dpi_x: 96,
                dpi_y: 96,
                scale_factor: 1.0,
                is_primary: true,
            },
            MonitorInfo {
                rc_monitor: Rect {
                    left: -2560,
                    top: 0,
                    right: 0,
                    bottom: 1600,
                },
                rc_work: Rect {
                    left: -2560,
                    top: 0,
                    right: 0,
                    bottom: 1560,
                },
                dpi_x: 144,
                dpi_y: 144,
                scale_factor: 1.5,
                is_primary: false,
            },
        ];

        // 位于左侧副屏的窗口
        let m = find_best_monitor(&monitors, -1000, 200, 300, 200).expect("Should find monitor");
        assert!(!m.is_primary);
        assert_eq!(m.scale_factor, 1.5);

        // 位于主屏的窗口
        let m2 = find_best_monitor(&monitors, 500, 200, 300, 200).expect("Should find monitor");
        assert!(m2.is_primary);
        assert_eq!(m2.scale_factor, 1.0);
    }

    #[test]
    fn test_clamp_to_work_area() {
        let monitor = MonitorInfo {
            rc_monitor: Rect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            rc_work: Rect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1040,
            },
            dpi_x: 96,
            dpi_y: 96,
            scale_factor: 1.0,
            is_primary: true,
        };

        // 窗口超出右边界
        let clamped = clamp_to_work_area(&monitor, 1800, 100, 300, 200);
        assert_eq!(clamped.0, 1620); // 1920 - 300
        assert_eq!(clamped.1, 100);

        // 窗口超出下边界
        let clamped_y = clamp_to_work_area(&monitor, 100, 1000, 300, 200);
        assert_eq!(clamped_y.0, 100);
        assert_eq!(clamped_y.1, 840); // 1040 - 200
    }

    #[test]
    fn bench_monitor_calculations() {
        let monitor = MonitorInfo {
            rc_monitor: Rect {
                left: 0,
                top: 0,
                right: 3840,
                bottom: 2160,
            },
            rc_work: Rect {
                left: 0,
                top: 0,
                right: 3840,
                bottom: 2100,
            },
            dpi_x: 192,
            dpi_y: 192,
            scale_factor: 2.0,
            is_primary: true,
        };

        let iterations = 50_000;
        let start = std::time::Instant::now();
        for i in 0..iterations {
            let _ = clamp_to_work_area(&monitor, i % 4000, (i * 2) % 2500, 400, 300);
        }
        let duration = start.elapsed();
        println!(
            "[性能测试] {iterations} 次高 DPI 屏幕边界限制计算耗时: {:?}",
            duration
        );
        assert!(duration.as_millis() < 50, "50000次边界计算应在50ms内完成");
    }
}
