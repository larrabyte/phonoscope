use gtk4::prelude::*;

fn main() {
    let app = gtk4::Application::builder()
        .application_id("dev.larrabyte.phonoscope")
        .build();

    app.connect_activate(move |app| {
        let player = mpris::PlayerFinder::new()
            .expect("Could not connect to D-Bus")
            .find_active()
            .expect("Could not find any player");

        let title = gtk4::Label::builder()
            .label("No song currently playing.")
            .selectable(true)
            .single_line_mode(true)
            .wrap(false)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .default_width(128)
            .default_height(72)
            .title("Phonoscope")
            .child(&title)
            .build();

        let refresh_interval = std::time::Duration::from_millis(100);
        glib::timeout_add_local(refresh_interval, move || {
            let metadata = player.get_metadata().unwrap();
            let name = metadata.title().unwrap_or("No song currently playing.");
            title.set_text(name);
            glib::Continue(true)
        });

        window.show();
    });

    app.run();
}
