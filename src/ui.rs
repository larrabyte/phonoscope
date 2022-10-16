use gtk4::prelude::*;

pub fn build_ui(app: &gtk4::Application) -> (gtk4::ApplicationWindow, gtk4::Label, gtk4::Label) {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let title = gtk4::Label::builder()
        .label("No song currently playing.")
        .single_line_mode(true)
        .wrap(false)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let lyrics = gtk4::Label::builder()
        .label("No lyrics available for display.")
        .single_line_mode(false)
        .wrap(false)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    container.append(&title);
    container.append(&lyrics);

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .default_width(128)
        .default_height(72)
        .title("Phonoscope")
        .child(&container)
        .build();

    (window, title, lyrics)
}
