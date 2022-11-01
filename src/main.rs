mod display;
mod selector;
mod warden;
mod parser;

use std::{convert::identity, time::Duration};
use selector::Selector;
use gtk::Orientation;
use display::Display;

use relm::prelude::*;
use adw::prelude::*;

pub struct Application {
    display: Controller<Display>
}

pub struct Widgets {
    title: adw::WindowTitle
}

#[derive(Debug)]
pub enum Event {
    Track(Option<String>),
    Poll(Duration),
    Reset
}

impl Component for Application {
    type CommandOutput = ();
    type Input = Event;
    type Output = ();
    type Init = ();
    type Root = adw::Window;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        adw::Window::builder()
            .default_width(800)
            .default_height(200)
            .build()
    }

    fn init(_: Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let content = gtk::Box::new(Orientation::Vertical, 0);
        let title = adw::WindowTitle::new("Phonoscope", "");
        let header = adw::HeaderBar::builder()
            .title_widget(&title)
            .build();

        // The window box (as in everything below the header).
        let window = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .vexpand(true)
            .build();

        let selector = Selector::builder()
            .launch(())
            .forward(sender.input_sender(), identity);

        let display = Display::builder()
            .launch(())
            .forward(sender.input_sender(), identity);

        content.append(&header);
        content.append(&window);

        window.append(display.widget());
        root.set_content(Some(&content));
        header.pack_start(selector.widget());

        let style = include_bytes!("style.css");
        relm::set_global_css(style);

        let model = Application {display};
        let widgets = Widgets {title};
        ComponentParts {model, widgets}
    }

    fn update_with_view(&mut self, widgets: &mut Self::Widgets, event: Self::Input, _: ComponentSender<Self>) {
        match event {
            Event::Track(Some(title)) => {
                widgets.title.set_title(&title);
                let message = display::Event::Initialise(title);
                self.display.emit(message);
            }

            Event::Poll(timestamp) => {
                let message = display::Event::Poll(timestamp);
                self.display.emit(message);
            }

            Event::Track(_) => widgets.title.set_title("No track currently playing."),
            Event::Reset => widgets.title.set_title("Phonoscope"),
        }
    }
}

fn main() {
    let app = RelmApp::new("dev.larrabyte.phonoscope");
    app.run::<Application>(());
}
