use gtk4 as gtk;

use gtk::gio;
use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use crate::utils::RenderPreset;

#[derive(Default)]
pub struct SpotlightWindow {
    pub entry: OnceCell<gtk::Entry>,
    pub store: OnceCell<gio::ListStore>,
    pub selection: OnceCell<gtk::SingleSelection>,
    pub list_view: OnceCell<gtk::ListView>,
    pub scroller: OnceCell<gtk::ScrolledWindow>,
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
        self.obj().build_ui();
    }
}

impl WidgetImpl for SpotlightWindow {}
impl WindowImpl for SpotlightWindow {}
impl ApplicationWindowImpl for SpotlightWindow {}
