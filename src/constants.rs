pub const DEFAULT_STYLES: &str = include_str!("../assets/default-styles.css");
pub const STYLE_FILE: &str = "style.css";
pub const JOTTO_LIB_CONFIG_DIR: &str = "jotto-utils";
pub const APP_CONFIG_DIR: &str = "spotlight";
pub const CONF_FILE_NAME: &str = "app.conf";
pub const DIR_START_SENTINEL: &str = "[dir start]";
pub const DIR_END_SENTINEL: &str = "[dir end]";

pub mod css_classes {
    pub const OVERLAY_ROOT: &str = "overlay-root";
    pub const OVERLAY_FILL: &str = "overlay-fill";
    pub const SEARCH_INPUT: &str = "search-input";
    pub const WINDOW_CONTENTS: &str = "window-content";
    pub const RESULT_SCROLLER: &str = "result-scroller";
    pub const RESULT_LIST: &str = "result-list";
    pub const RESULT_ITEM: &str = "result-item";
    pub const RESULT_ICON: &str = "result-icon";
}
