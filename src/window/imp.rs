use gtk::prelude::*;
use gtk4 as gtk;
use gtk4::glib::Properties;

use gtk::gio;
use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use crate::utils;
use crate::{error_fmt, error_log, error_log_exit};

#[derive(Properties, Default)]
#[properties(wrapper_type = super::SpotlightWindow)]
pub struct SpotlightWindow {
    pub entry: OnceCell<gtk::Entry>,
    pub store: OnceCell<gio::ListStore>,
    pub selection: OnceCell<gtk::SingleSelection>,
    pub list_view: OnceCell<gtk::ListView>,
    pub scroller: OnceCell<gtk::ScrolledWindow>,
    pub content: OnceCell<gtk::Box>,

    #[property(get, set, construct_only)]
    pub app_config: OnceCell<utils::AppConfig>,
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
        let Some(app_config) = self.app_config.get() else {
            error_log_exit!("Expected app_config at window constructed");
        };
        let Some(render_preset) = app_config.render_preset else {
            error_log_exit!("Expected render preset at window constructed");
        };
        self.obj().build_ui(render_preset);
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
