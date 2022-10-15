use gtk4::prelude::*;

fn main() {
    let app = gtk4::Application::builder()
        .application_id("dev.larrabyte.phonoscope")
        .build();

    app.connect_activate(|app| {
        // Create the main window.
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .default_width(640)
            .default_height(480)
            .title("Phonoscope")
            .build();

        window.show();
    });

    app.run();
}
