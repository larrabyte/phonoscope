use gtk4::{Application, ApplicationWindow, Label};

pub fn build_ui(app: &Application) -> (ApplicationWindow, Label, gtk4::Box) {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
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

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(128)
        .default_height(72)
        .title("Phonoscope")
        .child(&title)
        .child(&container)
        .build();

    (window, title, container)
}
