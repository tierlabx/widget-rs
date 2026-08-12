pub mod components;
pub mod layout;
pub mod main_window;
pub mod pages;
pub mod sidebar;
pub mod titlebar;
pub mod update;

pub use main_window::MainWindow;
pub use update::{check_for_update, download_update, MainWindowUpdateBridge, UpdateStatus};
