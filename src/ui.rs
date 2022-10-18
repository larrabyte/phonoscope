use gtk4::{Application, ApplicationWindow, Label};
use gtk4::prelude::*;

pub fn build_ui(app: &Application) -> (ApplicationWindow, Label, gtk4::Box) {
    let boss = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let title = gtk4::Label::builder()
        .label("No song currently playing.")
        .single_line_mode(true)
        .css_name("title")
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Start)
        .wrap(false)
        .margin_top(12)
        .margin_bottom(3)
        .margin_start(12)
        .margin_end(12)
        .build();

    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::End)
        .margin_top(12)
        .margin_bottom(3)
        .margin_start(12)
        .margin_end(12)
        .build();

    boss.append(&title);
    boss.append(&container);

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(128)
        .default_height(72)
        .title("Phonoscope")
        .child(&boss)
        .build();

    (window, title, container)
}
