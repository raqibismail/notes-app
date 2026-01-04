mod db;
use gtk4 as gtk;
use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    // Initialize database
    let _conn = db::setup_db().expect("Failed to initialize database");
    println!("Database is ready");

    // Initialize the application
    let app = gtk::Application::builder()
        .application_id("com.yourname.hyprnotes")
        .build();

    // Connect the "activate" signal
    app.connect_activate(|app| {
        // Create a horizontal box (Sidebar on left, Editor on right)
        let main_layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // Sidebar (where notes will be listed)
        let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar.set_size_request(200, -1); // Set a 200px width
        sidebar.append(&gtk::Label::new(Some("Notes List")));

        // Editor area (where you write)
        let editor = gtk::Box::new(gtk::Orientation::Vertical, 0);
        editor.set_hexpand(true); // Take up remaining horizontal space
        editor.append(&gtk::Label::new(Some("Editor Goes Here")));

        // Add both to main layout
        main_layout.append(&sidebar);
        main_layout.append(&editor);

        // Create the window
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("HyprNotes")
            .default_width(800)
            .default_height(600)
            .child(&main_layout)
            .build();

        window.present();
    });

    app.run()
}