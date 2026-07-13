mod imp;

use gtk4 as gtk;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, pango};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::rc::Rc;

use crate::constants::css_classes;
use crate::error_log;
use crate::model::AppObject;
use crate::modules::search;
use crate::utils;

glib::wrapper! {
    pub struct SpotlightWindow(ObjectSubclass<imp::SpotlightWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SpotlightWindow {
    pub fn new(
        app: &gtk::Application,
        app_config: utils::AppConfig,
        config: Rc<dir_search_rs::ParseConfig>,
    ) -> Self {
        let window: Self = glib::Object::builder().property("application", app).build();

        let imp = window.imp();
        let _ = imp.app_config.set(app_config);
        let _ = imp.config.set(config);

        window.run_search("");

        window
    }

    pub fn focus_entry(&self) {
        if let Some(entry) = self.imp().entry.get() {
            entry.grab_focus();
        }
    }

    pub fn run_search(&self, text: &str) {
        let imp = self.imp();
        let (Some(app_config), Some(config), Some(store), Some(list_view)) = (
            imp.app_config.get(),
            imp.config.get(),
            imp.store.get(),
            imp.list_view.get(),
        ) else {
            return;
        };
        let Some(preset) = app_config.render_preset else {
            return;
        };

        let items = {
            let mut last = imp.last_search_info.borrow_mut();
            search::run_search(preset, config, &mut last, text)
        };

        store.splice(0, store.n_items(), &items);

        let has_results = store.n_items() > 0;
        if let Some(scroller) = imp.scroller.get() {
            scroller.set_visible(has_results);
        }
        if has_results {
            list_view.scroll_to(0, gtk::ListScrollFlags::empty(), None);
        }
    }

    pub fn move_selection(&self, delta: i32) {
        let imp = self.imp();
        let (Some(selection), Some(list_view)) = (imp.selection.get(), imp.list_view.get()) else {
            return;
        };

        let count = selection.n_items();
        if count == 0 {
            return;
        }

        let current = selection.selected();
        let current = if current >= count { 0 } else { current };

        let next = if delta < 0 {
            current.saturating_sub(1)
        } else {
            (current + 1).min(count - 1)
        };

        selection.set_selected(next);
        list_view.scroll_to(next, gtk::ListScrollFlags::empty(), None);
    }

    pub fn launch_selected(&self) -> bool {
        let imp = self.imp();
        let (Some(selection), Some(app_config)) = (imp.selection.get(), imp.app_config.get())
        else {
            return false;
        };
        let Some(obj) = selection.selected_item().and_downcast::<AppObject>() else {
            return false;
        };

        match obj.launch(app_config.term.as_deref()) {
            Ok(()) => {
                if let Some(entry) = self.imp().entry.get() {
                    entry.delete_text(0, -1);
                }

                true
            }
            Err(err) => {
                error_log!(err);
                false
            }
        }
    }

    pub fn dismiss(&self) {
        if let Some(entry) = self.imp().entry.get() {
            entry.delete_text(0, -1);
        }

        if cfg!(debug_assertions) {
            self.close();
        } else {
            self.set_visible(false);
        }
    }

    fn build_ui(&self) {
        self.set_title(Some("Spotlight"));
        self.add_css_class(css_classes::OVERLAY_ROOT);

        self.setup_layer_shell();

        let content = self.build_content();
        let fill = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .hexpand(true)
            .vexpand(true)
            .css_classes([css_classes::OVERLAY_FILL])
            .build();
        fill.append(&content);
        self.set_child(Some(&fill));

        self.setup_actions();
        self.setup_dismiss_on_outside_click();
    }

    fn setup_layer_shell(&self) {
        self.init_layer_shell();
        self.set_layer(Layer::Overlay);
        self.set_anchor(Edge::Top, true);
        self.set_anchor(Edge::Bottom, true);
        self.set_anchor(Edge::Left, true);
        self.set_anchor(Edge::Right, true);
        self.set_keyboard_mode(KeyboardMode::Exclusive);
    }

    fn build_content(&self) -> gtk::Box {
        let imp = self.imp();

        let store = gio::ListStore::new::<AppObject>();
        let selection = gtk::SingleSelection::new(Some(store.clone()));

        let list_view = gtk::ListView::builder()
            .model(&selection)
            .factory(&build_factory())
            .single_click_activate(true)
            .css_classes([css_classes::RESULT_LIST])
            .single_click_activate(false)
            .build();

        list_view.connect_activate(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _| {
                if window.launch_selected() {
                    window.dismiss();
                }
            }
        ));

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .propagate_natural_height(true)
            .max_content_height(420)
            .css_classes([css_classes::RESULT_SCROLLER])
            .child(&list_view)
            .build();

        let entry = gtk::Entry::builder()
            .hexpand(true)
            .css_classes([css_classes::SEARCH_INPUT])
            .build();

        entry.connect_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |entry| {
                let query = entry.text();
                window.run_search(query.trim());
            }
        ));

        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .vexpand(true)
            .css_classes([css_classes::WINDOW_CONTENTS])
            .build();
        content.append(&entry);
        content.append(&scroller);

        let _ = imp.entry.set(entry);
        let _ = imp.store.set(store);
        let _ = imp.selection.set(selection);
        let _ = imp.list_view.set(list_view);
        let _ = imp.scroller.set(scroller);
        let _ = imp.content.set(content.clone());

        content
    }

    fn setup_actions(&self) {
        let actions = [
            gio::ActionEntry::builder("close")
                .activate(|window: &Self, _, _| window.dismiss())
                .build(),
            gio::ActionEntry::builder("select-prev")
                .activate(|window: &Self, _, _| window.move_selection(-1))
                .build(),
            gio::ActionEntry::builder("select-next")
                .activate(|window: &Self, _, _| window.move_selection(1))
                .build(),
            gio::ActionEntry::builder("launch-selected")
                .activate(|window: &Self, _, _| {
                    if window.launch_selected() {
                        window.dismiss();
                    }
                })
                .build(),
        ];
        self.add_action_entries(actions);
    }

    fn setup_dismiss_on_outside_click(&self) {
        let Some(content) = self.imp().content.get() else {
            return;
        };

        let click = gtk::GestureClick::new();
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        click.connect_pressed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[weak]
            content,
            move |_, _, x, y| {
                if let Some(bounds) = content.compute_bounds(&window)
                    && !bounds.contains_point(&gtk::graphene::Point::new(x as f32, y as f32))
                {
                    window.dismiss();
                }
            }
        ));
        self.add_controller(click);
    }
}

fn build_factory() -> gtk::SignalListItemFactory {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .css_classes([css_classes::RESULT_ITEM])
            .build();

        let icon = gtk::Image::builder()
            .pixel_size(28)
            .css_classes([css_classes::RESULT_ICON])
            .build();

        let label = gtk::Label::builder()
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::End)
            .build();

        row.append(&icon);
        row.append(&label);
        list_item.set_child(Some(&row));
    });

    factory.connect_bind(|_, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        let Some(obj) = list_item.item().and_downcast::<AppObject>() else {
            return;
        };
        let Some(row) = list_item.child().and_downcast::<gtk::Box>() else {
            return;
        };
        let Some(icon) = row.first_child().and_downcast::<gtk::Image>() else {
            return;
        };
        let Some(label) = icon.next_sibling().and_downcast::<gtk::Label>() else {
            return;
        };

        label.set_label(&obj.name());
        set_icon(&icon, obj.icon().as_deref());
    });

    factory
}

fn set_icon(image: &gtk::Image, icon: Option<&str>) {
    match icon {
        Some(name) if name.starts_with('/') => image.set_from_file(Some(name)),
        Some(name) => image.set_icon_name(Some(name)),
        None => image.set_icon_name(Some("application-x-executable")),
    }
}
