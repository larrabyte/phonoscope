use gtk::Orientation;

use std::{convert::identity, time::Duration};
use crate::parser::{Parser, Lyric};
use relm::WorkerController;
use crate::parser;
use gtk::Align;
use std::io;

use relm::prelude::*;
use adw::prelude::*;

pub struct Display {
    parser: WorkerController<Parser>,
    cached: Duration,
    lyrics: Vec<Lyric>
}

pub struct Widgets {
    root: gtk::Box,
    children: Vec<gtk::Box>
}

#[derive(Debug)]
pub enum Event {
    Initialise(String),
    IOFailed(io::Error),
    ParseFailed(parser::Error),
    Success(Vec<Lyric>),
    Poll(Duration)
}

impl Widgets {
    fn clear(&mut self) {
        self.children.iter().for_each(|r| self.root.remove(r));
        self.children.clear();
    }
}

impl Component for Display {
    type CommandOutput = ();
    type Init = ();
    type Input = Event;
    type Output = crate::Event;
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        gtk::Box::new(Orientation::Horizontal, 0)
    }

    fn init(_: Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let parser = Parser::builder()
            .detach_worker(())
            .forward(sender.input_sender(), identity);

        let model = Display {parser, cached: Duration::default(), lyrics: Vec::new()};

        let widgets = Widgets {
            // Cloning gtk::Box doesn't seem to clone the actual box, but rather
            // just creates a new (non-referential) object. Rc<gtk::Box>?
            root: root.clone(),
            children: Vec::new()
        };

        ComponentParts {model, widgets}
    }

    fn update_with_view(&mut self, widgets: &mut Self::Widgets, event: Self::Input, _: ComponentSender<Self>) {
        match event {
            Event::Initialise(title) => {
                self.lyrics.clear();
                let message = parser::Event::Parse(title);
                self.parser.emit(message);
            }

            Event::IOFailed(err) => {
                widgets.clear();

                let wrapper = gtk::Box::builder()
                    .orientation(Orientation::Vertical)
                    .spacing(12)
                    .build();

                let message = gtk::Label::builder()
                    .css_name("characters")
                    .label("Failed to read lyrics file.")
                    .build();

                let description = gtk::Label::builder()
                    .css_name("error-description")
                    .label(&format!("{:?}", err))
                    .build();

                wrapper.append(&message);
                wrapper.append(&description);
                widgets.root.append(&wrapper);
                widgets.children.push(wrapper);
            }

            Event::ParseFailed(err) => {
                widgets.clear();

                let wrapper = gtk::Box::builder()
                    .orientation(Orientation::Vertical)
                    .spacing(12)
                    .build();

                let message = gtk::Label::builder()
                    .css_name("characters")
                    .label("Failed to parse lyrics file.")
                    .build();

                let description = gtk::Label::builder()
                    .css_name("error-description")
                    .label(&format!("{:?}", err))
                    .build();

                wrapper.append(&message);
                wrapper.append(&description);
                widgets.root.append(&wrapper);
                widgets.children.push(wrapper);
            }

            Event::Success(lyrics) => {
                self.lyrics = lyrics;
                widgets.clear();
            }

            Event::Poll(timestamp) => {
                if let Some(lyric) = self.lyrics.iter().filter(|l| l.timestamp < timestamp).last() {
                    if self.cached != lyric.timestamp {
                        widgets.clear();

                        self.cached = lyric.timestamp;

                        for ruby in &lyric.rubies {
                            let wrapper = gtk::Box::builder()
                                .orientation(Orientation::Vertical)
                                .valign(Align::End)
                                .build();
    
                            let reading = gtk::Label::builder()
                                .css_name("reading")
                                .label(ruby.reading.as_deref().unwrap_or(""))
                                .build();

                            // Since whitespace characters are parsed to empty strings, we convert them when displaying.
                            let characters = if ruby.characters.is_empty() {" "} else {&ruby.characters};

                            let characters = gtk::Label::builder()
                                .css_name("characters")
                                .label(characters)
                                .build();
    
                            wrapper.append(&reading);
                            wrapper.append(&characters);
                            widgets.root.append(&wrapper);
                            widgets.children.push(wrapper);
                        }
                    }
                }
            }
        }
    }
}
