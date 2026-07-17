use gtk::prelude::*;
use gtk4 as gtk;
use gtk4::glib::Properties;

use gtk::{gio, glib, subclass::prelude::*};
use std::cell::{Cell, OnceCell, RefCell};

use crate::modules::config;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::SpotlightWindow)]
pub struct SpotlightWindow {
    pub entry: OnceCell<gtk::Entry>,
    pub store: OnceCell<gio::ListStore>,
    pub selection: OnceCell<gtk::SingleSelection>,
    pub list_view: OnceCell<gtk::ListView>,
    pub scroller: OnceCell<gtk::ScrolledWindow>,
    pub content: OnceCell<gtk::Box>,
    pub math_revealer: OnceCell<gtk::Revealer>,

    #[property(get, set, construct_only)]
    pub app_config: RefCell<config::AppConfig>,
    pub config: RefCell<dir_search_rs::ParseConfig>,
    pub last_search_info: RefCell<Option<dir_search_rs::LastRunInfo>>,
    pub cli_connection: Cell<Option<gio::ApplicationCommandLine>>,
    pub root_instance: Cell<bool>,
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
        let app_config = self.app_config.borrow();
        self.obj().build_ui(app_config.render_preset);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        Self::derived_properties()
    }

    fn set_property(&self, id: usize, value: &glib::Value, param_spec: &glib::ParamSpec) {
        self.derived_set_property(id, value, param_spec)
    }

    fn property(&self, id: usize, param_spec: &glib::ParamSpec) -> glib::Value {
        self.derived_property(id, param_spec)
    }
}

impl WidgetImpl for SpotlightWindow {}
impl WindowImpl for SpotlightWindow {}
impl ApplicationWindowImpl for SpotlightWindow {}
