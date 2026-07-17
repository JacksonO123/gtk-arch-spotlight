mod imp;

use gio::glib::property::PropertySet;
use gtk4 as gtk;

use gtk::{gdk, gio, glib, pango, prelude::*, subclass::prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    constants::{self, css_classes},
    error_fmt, error_log,
    model::AppObject,
    modules::{config, parser, search},
};

glib::wrapper! {
    pub struct SpotlightWindow(ObjectSubclass<imp::SpotlightWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SpotlightWindow {
    pub fn new(
        app: &gtk::Application,
        app_config: config::AppConfig,
        config: dir_search_rs::ParseConfig,
        is_root_instance: bool,
    ) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .property("app_config", app_config)
            .build();

        let imp = window.imp();
        imp.config.set(config);
        imp.root_instance.set(is_root_instance);

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
        let (app_config, config, Some(store), Some(list_view)) = (
            imp.app_config.borrow(),
            imp.config.borrow(),
            imp.store.get(),
            imp.list_view.get(),
        ) else {
            return;
        };

        let items = {
            let mut last = imp.last_search_info.borrow_mut();
            search::run_search(app_config.render_preset, &config, &mut last, text)
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

    pub fn try_calculate_math(&self, expr_str: &str) {
        let imp = self.imp();
        let Some(math_revealer) = imp.math_revealer.get() else {
            return;
        };
        let Some(label_wrapper) = math_revealer.child() else {
            return;
        };

        let math_res = parser::evaluate_str(expr_str);
        let mut show_math = false;
        let math_str = if let Ok(Some(result)) = math_res {
            show_math = true;
            &result.to_string()
        } else {
            "Err"
        };
        show_math = show_math || parser::contains_operator(expr_str);

        if show_math {
            let Some(label_child) = label_wrapper.first_child().and_downcast::<gtk::Label>() else {
                return;
            };

            label_child.set_label(math_str);
            math_revealer.set_reveal_child(true);
            label_wrapper.remove_css_class(css_classes::TRANSPARENT);
        } else {
            math_revealer.set_reveal_child(false);
            label_wrapper.add_css_class(css_classes::TRANSPARENT);
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
        let (Some(selection), app_config) = (imp.selection.get(), imp.app_config.borrow()) else {
            return false;
        };
        let Some(obj) = selection.selected_item().and_downcast::<AppObject>() else {
            return false;
        };

        let cli_connection = imp.cli_connection.take();
        let is_root_instance = imp.root_instance.get();

        match obj.launch(app_config.term.as_deref()) {
            Ok(path) => {
                if let Some(entry) = self.imp().entry.get() {
                    entry.delete_text(0, -1);
                }

                if let Some(path) = path
                    && app_config.write_stdout
                    && let Some(cmd) = cli_connection
                {
                    cmd.print_literal(&format!("{}\n", path.display()));
                    cmd.set_exit_code(glib::ExitCode::SUCCESS);
                    if is_root_instance {
                        std::process::exit(0);
                    }
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
        if let Some(cmd) = self.imp().cli_connection.take() {
            cmd.set_exit_code(glib::ExitCode::FAILURE);
        }

        if let Some(entry) = self.imp().entry.get() {
            entry.delete_text(0, -1);
        }

        if cfg!(debug_assertions) {
            self.close();
        } else {
            self.set_visible(false);
        }
    }

    pub fn set_list_factory(&self, factory: gtk::SignalListItemFactory) {
        let imp = self.imp();
        let Some(list_view) = imp.list_view.get() else {
            return;
        };

        list_view.set_factory(Some(&factory));
    }

    fn build_ui(&self, render_preset: config::RenderPreset) {
        self.set_title(Some("Spotlight"));
        self.add_css_class(css_classes::OVERLAY_ROOT);

        self.setup_layer_shell();

        let content = self.build_content(render_preset);
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

    fn build_content(&self, render_preset: config::RenderPreset) -> gtk::Box {
        let imp = self.imp();

        let store = gio::ListStore::new::<AppObject>();
        let selection = gtk::SingleSelection::new(Some(store.clone()));

        let list_view = gtk::ListView::builder()
            .model(&selection)
            .factory(&build_factory(
                render_preset,
                ImageCache::new(constants::IMAGE_CACHE_CAP),
            ))
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
            .max_content_height(600)
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
                let text = query.trim();
                window.run_search(text);
                window.try_calculate_math(text);
            }
        ));

        let revealer_label = gtk::Label::builder()
            .css_classes([css_classes::MATH_LABEL])
            .hexpand(true)
            .build();
        let revealer_label_wrapper = gtk::Box::builder()
            .css_classes([css_classes::MATH_LABEL_WRAPPER])
            .hexpand(true)
            .build();
        revealer_label_wrapper.append(&revealer_label);
        let math_revealer = gtk::Revealer::builder()
            .child(&revealer_label_wrapper)
            .transition_duration(constants::ANIMATION_DURATION)
            .transition_type(gtk4::RevealerTransitionType::SlideUp)
            .reveal_child(false)
            .build();

        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .vexpand(true)
            .css_classes([css_classes::WINDOW_CONTENTS])
            .build();
        content.append(&entry);
        content.append(&math_revealer);
        content.append(&scroller);

        _ = imp.entry.set(entry);
        _ = imp.store.set(store);
        _ = imp.selection.set(selection);
        _ = imp.list_view.set(list_view);
        _ = imp.scroller.set(scroller);
        _ = imp.math_revealer.set(math_revealer);
        _ = imp.content.set(content.clone());

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

pub fn build_factory(
    render_preset: config::RenderPreset,
    image_cache: ImageCache,
) -> gtk::SignalListItemFactory {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(move |_, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .css_classes([css_classes::RESULT_ITEM])
            .build();

        let icon = gtk::Image::builder()
            .pixel_size(if render_preset == config::RenderPreset::DesktopFile {
                28
            } else {
                constants::IMAGE_ICON_SIZE
            })
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

    factory.connect_bind(move |_, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        bind_list_item(list_item, render_preset, &image_cache);
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

fn bind_list_item(
    list_item: &gtk::ListItem,
    render_preset: config::RenderPreset,
    image_cache: &ImageCache,
) {
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

    match render_preset {
        config::RenderPreset::DesktopFile => {
            let Some(name) = &obj.name() else {
                return;
            };
            label.set_label(name);
            set_icon(&icon, obj.icon().as_deref());
        }
        config::RenderPreset::Images => {
            let path = obj.get_img_path();
            let Some(path) = path else {
                return;
            };
            if let Some(label_str) = path.iter().next_back() {
                label.set_label(label_str.to_str().unwrap());
            }
            bind_image(&icon, path, image_cache);
        }
        config::RenderPreset::None => {}
    }
}

fn bind_image(icon: &gtk::Image, path: PathBuf, image_cache: &ImageCache) {
    let path_name = path.to_string_lossy().into_owned();
    icon.set_widget_name(&path_name);

    if let Some(texture) = image_cache.get(&path) {
        icon.set_paintable(Some(&texture));
        return;
    }

    icon.set_icon_name(Some("image-x-generic"));

    let icon_weak = icon.downgrade();
    let cache = image_cache.clone();
    glib::spawn_future_local(async move {
        let load_path = path.clone();
        let decoded = gio::spawn_blocking(move || {
            decode_scaled_image(&load_path, constants::IMAGE_ICON_SIZE)
        })
        .await;
        let Ok(Some(decoded)) = decoded else {
            return;
        };

        let texture = texture_from_decoded(decoded);
        cache.insert(path, texture.clone());

        if let Some(icon) = icon_weak.upgrade()
            && icon.widget_name().as_str() == path_name.as_str()
        {
            icon.set_paintable(Some(&texture));
        }
    });
}

struct DecodedImage {
    bytes: Vec<u8>,
    width: i32,
    height: i32,
    rowstride: i32,
    has_alpha: bool,
}

fn decode_scaled_image(path: &Path, size: i32) -> Option<DecodedImage> {
    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(path, size, size, true).ok()?;
    Some(DecodedImage {
        width: pixbuf.width(),
        height: pixbuf.height(),
        rowstride: pixbuf.rowstride(),
        has_alpha: pixbuf.has_alpha(),
        bytes: pixbuf.read_pixel_bytes().to_vec(),
    })
}

fn texture_from_decoded(img: DecodedImage) -> gdk::Texture {
    let format = if img.has_alpha {
        gdk::MemoryFormat::R8g8b8a8
    } else {
        gdk::MemoryFormat::R8g8b8
    };
    let bytes = glib::Bytes::from_owned(img.bytes);
    gdk::MemoryTexture::new(
        img.width,
        img.height,
        format,
        &bytes,
        img.rowstride as usize,
    )
    .upcast()
}

#[derive(Clone)]
pub struct ImageCache {
    inner: Rc<RefCell<ImageCacheInner>>,
}

struct ImageCacheInner {
    map: HashMap<PathBuf, gdk::Texture>,
    order: VecDeque<PathBuf>,
    cap: usize,
}

impl ImageCache {
    pub fn new(cap: usize) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ImageCacheInner {
                map: HashMap::new(),
                order: VecDeque::new(),
                cap,
            })),
        }
    }

    fn get(&self, path: &Path) -> Option<gdk::Texture> {
        self.inner.borrow().map.get(path).cloned()
    }

    fn insert(&self, path: PathBuf, texture: gdk::Texture) {
        let mut inner = self.inner.borrow_mut();
        if inner.map.contains_key(&path) {
            return;
        }
        while inner.map.len() >= inner.cap {
            let Some(old) = inner.order.pop_front() else {
                break;
            };
            inner.map.remove(&old);
        }
        inner.order.push_back(path.clone());
        inner.map.insert(path, texture);
    }
}
