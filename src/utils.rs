use crate::constants::{APP_CONFIG_DIR, DEFAULT_STYLES, JOTTO_LIB_CONFIG_DIR, STYLE_FILE};
use gtk::{gdk, gio, glib};
use gtk4 as gtk;

pub fn load_css() {
    let mut config_path = glib::user_config_dir();
    config_path.push(JOTTO_LIB_CONFIG_DIR);
    config_path.push(APP_CONFIG_DIR);
    config_path.push(STYLE_FILE);

    let default_display = &gdk::Display::default().expect("Could not connect to a display");

    if config_path.exists() {
        let provider = gtk::CssProvider::new();
        let gio_file = gio::File::for_path(config_path);
        provider.load_from_file(&gio_file);
        gtk::style_context_add_provider_for_display(
            default_display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_data(DEFAULT_STYLES);
    gtk::style_context_add_provider_for_display(
        default_display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
