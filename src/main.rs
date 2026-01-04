mod db;
use gtk::prelude::*;
use gtk4 as gtk;
use std::sync::{Arc, Mutex};

fn main() -> gtk::glib::ExitCode {
    // Initialize database
    let conn = db::setup_db().expect("Failed to initialize database");
    println!("Database is ready");
    let shared_conn = Arc::new(Mutex::new(conn));

    // Initialize the application
    let app = gtk::Application::builder()
        .application_id("com.yourname.hyprnotes")
        .build();

    // Connect the "activate" signal
    app.connect_activate(move |app| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("style.css"));

        // Apply the CSS to the whole screen
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // 1. The Main Horizontal Split
        let main_layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // 2. Left Side: Sidebar
        let sidebar_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar_container.set_size_request(250, -1);
        sidebar_container.add_css_class("sidebar"); // We can style this later

        let scrolled_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never) // No horizontal scroll
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let list_box = gtk::ListBox::new();
        scrolled_window.set_child(Some(&list_box));

        // A "New Note" button at the bottom of sidebar
        let btn_new = gtk::Button::with_label(" + New Note ");
        btn_new.connect_clicked(move |_| {
            println!("New Note button clicked");
        });
        btn_new.set_margin_top(10);
        btn_new.set_margin_bottom(10);
        btn_new.set_margin_start(10);
        btn_new.set_margin_end(10);

        sidebar_container.append(&scrolled_window);
        sidebar_container.append(&btn_new);

        // 3. Right Side: Editor
        let editor_container = gtk::Box::new(gtk::Orientation::Vertical, 5);
        editor_container.set_hexpand(true);
        editor_container.set_margin_top(20);
        editor_container.set_margin_bottom(20);
        editor_container.set_margin_start(20);
        editor_container.set_margin_end(20);

        let title_entry = gtk::Entry::builder().placeholder_text("Note Title").build();
        let text_view = gtk::TextView::builder()
            .vexpand(true)
            .wrap_mode(gtk::WrapMode::Word)
            .build();

        let btn_save = gtk::Button::with_label("Save Note");

        // --- FIX STARTS HERE ---
        let title_clone = title_entry.clone();
        let text_view_for_save = text_view.clone(); // Clone specifically for the save button
        let conn_clone = shared_conn.clone();

        btn_save.connect_clicked(move |_| {
            let title = title_clone.text().to_string();

            // Use the specific clone we made for this closure
            let buffer = text_view_for_save.buffer();
            let (start, end) = buffer.bounds();
            let content = buffer.text(&start, &end, false).to_string();

            let conn = conn_clone.lock().unwrap();
            match db::insert_note(&conn, &title, &content) {
                Ok(_) => println!("Saved: {}", title),
                Err(e) => eprintln!("Error: {}", e),
            }
        });

        // Now we use the ORIGINAL text_view for the UI layout.
        // Since it wasn't moved into the closure, this works!
        let editor_scroll = gtk::ScrolledWindow::new();
        editor_scroll.set_child(Some(&text_view));
        // ------------------------

        editor_container.append(&title_entry);
        editor_container.append(&editor_scroll);
        editor_container.append(&btn_save);

        // 4. Putting it all together
        main_layout.append(&sidebar_container);
        // Add a separator for that "Windows" look
        main_layout.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        main_layout.append(&editor_container);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("HyprNotes")
            .default_width(900)
            .default_height(600)
            .child(&main_layout)
            .build();

        window.present();
    });

    app.run()
}
