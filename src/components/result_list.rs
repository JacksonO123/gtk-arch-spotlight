use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{gio, pango};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app_state;
use crate::constants::css_classes;
use crate::error_log;
use crate::model::AppObject;
use crate::modules::search;

/// Cheap-to-clone handles to the list's model and view.
///
/// Each field is a `GObject`, so cloning is just a refcount bump. These are
/// passed to the search entry (to repopulate) and to the key/activate handlers
/// (to move the selection and launch).
#[derive(Clone)]
pub struct ListHandles {
    pub store: gio::ListStore,
    pub selection: gtk::SingleSelection,
    pub list_view: gtk::ListView,
}

pub fn create_element(
    the_app_state: &Rc<RefCell<app_state::AppState>>,
    config: &Rc<dir_search_rs::ParseConfig>,
) -> (gtk::ScrolledWindow, ListHandles) {
    let store = gio::ListStore::new::<AppObject>();
    // `SingleSelection` defaults to autoselect + can't-unselect, so the first
    // result is always highlighted after a search with no extra bookkeeping.
    let selection = gtk::SingleSelection::new(Some(store.clone()));

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

    let list_view = gtk::ListView::builder()
        .model(&selection)
        .factory(&factory)
        .single_click_activate(true)
        .css_classes([css_classes::RESULT_LIST])
        .build();

    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        // Grow to fit the results, but cap the height and scroll beyond that.
        .propagate_natural_height(true)
        .max_content_height(420)
        .css_classes([css_classes::RESULT_SCROLLER])
        .child(&list_view)
        .build();

    let handles = ListHandles {
        store,
        selection,
        list_view,
    };

    // Seed the list with the empty-query results.
    populate(&handles, &mut the_app_state.borrow_mut(), config, "");

    (scroller, handles)
}

/// Re-run the search for `text` and replace the list contents.
pub fn populate(
    handles: &ListHandles,
    state: &mut app_state::AppState,
    config: &dir_search_rs::ParseConfig,
    text: &str,
) {
    let items = search::run_search(state, config, text);
    handles
        .store
        .splice(0, handles.store.n_items(), &items);

    if handles.store.n_items() > 0 {
        handles
            .list_view
            .scroll_to(0, gtk::ListScrollFlags::empty(), None);
    }
}

/// Move the selection by `delta` rows, keeping it in bounds and scrolling the
/// newly-selected row into view.
pub fn move_selection(handles: &ListHandles, delta: i32) {
    let count = handles.selection.n_items();
    if count == 0 {
        return;
    }

    let current = handles.selection.selected();
    // `selected()` is GTK_INVALID_LIST_POSITION (u32::MAX) when nothing is set.
    let current = if current >= count { 0 } else { current };

    let next = if delta < 0 {
        current.saturating_sub(1)
    } else {
        (current + 1).min(count - 1)
    };

    handles.selection.set_selected(next);
    handles
        .list_view
        .scroll_to(next, gtk::ListScrollFlags::empty(), None);
}

/// Launch the currently selected application. Returns `true` on success.
pub fn launch_selected(handles: &ListHandles) -> bool {
    let Some(obj) = handles.selection.selected_item().and_downcast::<AppObject>() else {
        return false;
    };

    if let Err(err) = obj.launch() {
        error_log!(err);
        return false;
    }

    true
}

fn set_icon(image: &gtk::Image, icon: Option<&str>) {
    match icon {
        Some(name) if name.starts_with('/') => image.set_from_file(Some(name)),
        Some(name) => image.set_icon_name(Some(name)),
        None => image.set_icon_name(Some("application-x-executable")),
    }
}
