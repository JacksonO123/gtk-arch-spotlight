pub const DEFAULT_STYLES: &str = include_str!("../assets/default-styles.css");
pub const STYLE_FILE: &str = "style.css";
pub const JOTTO_LIB_CONFIG_DIR: &str = "jotto-utils";
pub const APP_CONFIG_DIR: &str = "spotlight";
pub const ANIMATION_DURATION_MS: u32 = 250;
pub const MAX_RESULTS: usize = 8;

pub mod css_classes {
    pub const RESULT_ITEM: &str = "result-item";
    pub const OVERLAY_ROOT: &str = "overlay-root";
    pub const OVERLAY_FILL: &str = "overlay-fill";
    pub const SEARCH_INPUT: &str = "search-input";
    pub const WINDOW_CONTENTS: &str = "window-content";
    pub const RESULT_WRAPPER: &str = "result-wrapper";
    pub const ACTIVE_RESULT: &str = "active-result";
    pub const EMPTY: &str = "empty";
    pub const TRANSITION_ALL: &str = "transition-all";
}
