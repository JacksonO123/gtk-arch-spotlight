use gtk4 as gtk;

use gtk::gio;
use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use crate::utils::RenderPreset;

/// Private state for [`super::SpotlightWindow`].
///
/// Everything the window needs lives here — the widget handles built during
/// construction and the search state. This replaces the old
/// `Rc<RefCell<AppState>>`: mutable state is held in the subclass instance
/// itself, which is the idiomatic gtk4-rs approach.
#[derive(Default)]
pub struct SpotlightWindow {
    pub entry: OnceCell<gtk::Entry>,
    pub store: OnceCell<gio::ListStore>,
    pub selection: OnceCell<gtk::SingleSelection>,
    pub list_view: OnceCell<gtk::ListView>,
    pub content: OnceCell<gtk::Box>,

    pub render_preset: OnceCell<RenderPreset>,
    pub config: OnceCell<Rc<dir_search_rs::ParseConfig>>,
    pub last_search_info: RefCell<Option<dir_search_rs::LastRunInfo>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SpotlightWindow {
    const NAME: &'static str = "SpotlightWindow";
    type Type = super::SpotlightWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for SpotlightWindow {
    fn constructed(&self) {
        self.parent_constructed();
        // Build the widget tree, actions and layer-shell config. Search-time
        // state (preset/config) is populated by `SpotlightWindow::new` after
        // construction, so nothing here may depend on it.
        self.obj().build_ui();
    }
}

impl WidgetImpl for SpotlightWindow {}
impl WindowImpl for SpotlightWindow {}
impl ApplicationWindowImpl for SpotlightWindow {}
