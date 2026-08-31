use gpui::*;

/// 获取主进程私有物理内存占用（bytes）
pub fn get_private_memory_usage() -> usize {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::ProcessStatus::K32GetProcessMemoryInfo;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        #[allow(non_snake_case)]
        #[repr(C)]
        struct PROCESS_MEMORY_COUNTERS_EX2 {
            pub cb: u32,
            pub PageFaultCount: u32,
            pub PeakWorkingSetSize: usize,
            pub WorkingSetSize: usize,
            pub QuotaPeakPagedPoolUsage: usize,
            pub QuotaPagedPoolUsage: usize,
            pub QuotaPeakNonPagedPoolUsage: usize,
            pub QuotaNonPagedPoolUsage: usize,
            pub PagefileUsage: usize,
            pub PeakPagefileUsage: usize,
            pub PrivateUsage: usize,
            pub PrivateWorkingSetSize: usize,
            pub SharedCommitUsage: u64,
        }

        unsafe {
            let mut mem_counters: PROCESS_MEMORY_COUNTERS_EX2 = std::mem::zeroed();
            mem_counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX2>() as u32;
            if K32GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut mem_counters as *mut _ as *mut _,
                mem_counters.cb,
            ) != 0
            {
                return mem_counters.PrivateWorkingSetSize;
            }
        }
    }

    memory_stats::memory_stats()
        .map(|s| s.physical_mem)
        .unwrap_or(0)
}

/// 渲染控制面板顶部单个指标统计卡片
pub fn render_stat_card(
    icon: gpui_component::IconName,
    num: impl Into<SharedString>,
    label: &'static str,
    ic: Rgba,
    bg: Rgba,
    border: Rgba,
) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .items_center()
        .gap(px(10.0))
        .px(px(16.0))
        .py(px(12.0))
        .bg(bg)
        .border_1()
        .border_color(border)
        .rounded(px(8.0))
        .child(div().text_color(ic).child(gpui_component::Icon::new(icon)))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xf2f2f2))
                        .child(num.into()),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(ic)
                        .child(label),
                ),
        )
}
