mod db;

use gtk::prelude::*;
use gtk4 as gtk;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// 1. Define a State Struct to hold your widgets and shared data
#[derive(Clone)]
struct AppState {
    conn: Arc<Mutex<rusqlite::Connection>>,
    current_note_id: Rc<RefCell<Option<i32>>>,
    title_entry: gtk::Entry,
    text_view: gtk::TextView,
    list_box: gtk::ListBox,
}

fn main() -> gtk::glib::ExitCode {
    let conn = db::setup_db().expect("Failed to initialize database");
    let shared_conn = Arc::new(Mutex::new(conn));

    let app = gtk::Application::builder()
        .application_id("com.yourname.hyprnotes")
        .build();

    app.connect_activate(move |obj_app| {
        setup_styles();
        build_ui(obj_app, &shared_conn);
    });

    app.run()
}

// 2. Separate CSS styling
fn setup_styles() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

// 3. Centralized UI Constructor
fn build_ui(app: &gtk::Application, conn: &Arc<Mutex<rusqlite::Connection>>) {
    let title_entry = gtk::Entry::builder().placeholder_text("Note Title").build();
    let text_view = gtk::TextView::builder()
        .vexpand(true)
        .wrap_mode(gtk::WrapMode::Word)
        .build();
    let list_box = gtk::ListBox::new();

    let state = AppState {
        conn: conn.clone(),
        current_note_id: Rc::new(RefCell::new(None)),
        title_entry,
        text_view,
        list_box,
    };

    let main_layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_layout.append(&build_sidebar(&state));
    main_layout.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    main_layout.append(&build_editor(&state));

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("HyprNotes")
        .default_width(900)
        .default_height(600)
        .child(&main_layout)
        .build();

    refresh_list(&state.list_box, &state.conn);
    window.present();
}

// 4. Component Builders
fn build_sidebar(state: &AppState) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.set_size_request(250, -1);
    container.add_css_class("sidebar");

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&state.list_box)
        .build();

    let btn_new = gtk::Button::with_label(" + New Note ");
    btn_new.set_margin_start(10);
    btn_new.set_margin_end(10);
    btn_new.set_margin_top(10);
    btn_new.set_margin_bottom(10);

    let s = state.clone();
    btn_new.connect_clicked(move |_| handle_new_note(&s));

    let s = state.clone();
    state.list_box.connect_row_selected(move |_, row| {
        if let Some(r) = row {
            handle_row_selected(&s, r);
        }
    });

    container.append(&scrolled);
    container.append(&btn_new);
    container
}

fn build_editor(state: &AppState) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    container.set_margin_top(20);
    container.set_margin_bottom(20);
    container.set_margin_start(20);
    container.set_margin_end(20);
    container.set_hexpand(true);

    // 1. Give the scrolled window a specific class for CSS targeting
    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&state.text_view)
        .build();
    scrolled.add_css_class("editor-scroll");

    // 2. Internal Text Inset (The "Fix" for flushed text)
    state.text_view.set_left_margin(15);
    state.text_view.set_right_margin(15);
    state.text_view.set_top_margin(15);
    state.text_view.set_bottom_margin(15);

    // 3. Align buttons to the RIGHT
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    button_box.set_halign(gtk::Align::End); // <--- This pushes buttons to the right

    let btn_save = gtk::Button::with_label("Save Note");
    btn_save.add_css_class("suggested-action");
    let s_save = state.clone();
    btn_save.connect_clicked(move |_| handle_save(&s_save));

    let btn_delete = gtk::Button::with_label("Delete Note");
    btn_delete.add_css_class("destructive-action");
    let s_delete = state.clone();
    btn_delete.connect_clicked(move |_| handle_delete(&s_delete));

    button_box.append(&btn_save);
    button_box.append(&btn_delete);

    container.append(&state.title_entry);
    container.append(&scrolled);
    container.append(&button_box);
    container
}

// 5. Logical Handlers
fn handle_new_note(state: &AppState) {
    *state.current_note_id.borrow_mut() = None;
    state.title_entry.set_text("");
    state.text_view.buffer().set_text("");
}

fn handle_row_selected(state: &AppState, row: &gtk::ListBoxRow) {
    let note_id_str = row.widget_name();
    if note_id_str.is_empty() {
        return;
    }

    let note_id: i32 = note_id_str.parse().unwrap_or(0);

    // 1. Use try_borrow_mut instead of borrow_mut to prevent panics
    if let Ok(mut id_borrow) = state.current_note_id.try_borrow_mut() {
        *id_borrow = Some(note_id);
    } else {
        // If we can't borrow, it means a refresh is likely happening.
        // We return early to avoid the crash.
        return;
    }

    let conn = state.conn.lock().unwrap();
    if let Ok(note) = db::get_note_by_id(&conn, note_id) {
        state.title_entry.set_text(&note.title);
        state.text_view.buffer().set_text(&note.content);
    }
}

fn handle_save(state: &AppState) {
    let title = state.title_entry.text().to_string();
    let buffer = state.text_view.buffer();
    let content = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string();
    let current_id = *state.current_note_id.borrow();

    let res = {
        let conn = state.conn.lock().unwrap();
        match current_id {
            Some(id) => db::update_note(&conn, id, &title, &content),
            None => db::insert_note(&conn, &title, &content),
        }
    };

    if res.is_ok() {
        *state.current_note_id.borrow_mut() = None;
        refresh_list(&state.list_box, &state.conn);
    }
}

fn handle_delete(state: &AppState) {
    // Scope the ID extraction
    let id_to_delete = {
        let id_borrow = state.current_note_id.borrow();
        *id_borrow
    }; // Borrow ends here

    if let Some(id) = id_to_delete {
        let success = {
            let conn = state.conn.lock().unwrap();
            db::delete_note(&conn, id).is_ok()
        };

        if success {
            handle_new_note(state);
            // Now refresh_list is safe because current_note_id is NOT borrowed
            refresh_list(&state.list_box, &state.conn);
        }
    }
}

fn refresh_list(list_box: &gtk::ListBox, conn: &Arc<Mutex<rusqlite::Connection>>) {
    // Get data first, then drop the lock
    let notes = {
        let conn_lock = conn.lock().unwrap();
        db::get_all_notes(&conn_lock).unwrap_or_default()
    };

    // Clear and rebuild UI
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    for note in notes {
        let row = gtk::ListBoxRow::builder()
            .child(
                &gtk::Label::builder()
                    .label(&note.title)
                    .xalign(0.0)
                    .margin_start(10)
                    .margin_end(10)
                    .margin_top(5)
                    .margin_bottom(5)
                    .build(),
            )
            .name(note.id.to_string())
            .build();
        list_box.append(&row);
    }
}
