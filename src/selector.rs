use relm::WorkerController;
use std::convert::identity;
use mpris::FindingError;
use gtk::SelectionMode;
use glib::clone;

use crate::warden::{Warden, Endpoint};
use crate::warden;

use relm::prelude::*;
use adw::prelude::*;

pub struct Selector {
    warden: WorkerController<Warden>
}

pub struct Widgets {
    popover: gtk::Popover,
    listbox: gtk::ListBox,
    rows: Vec<adw::ActionRow>
}

#[derive(Debug)]
pub enum Event {
    Button,
    Row(usize),
    Bubble(crate::Event),
    Failed(FindingError),
    Fetched(Vec<Endpoint>)
}

impl Widgets {
    fn clear(&mut self) {
        self.rows.iter().for_each(|r| self.listbox.remove(r));
        self.rows.clear();
    }
}

impl Component for Selector {
    type CommandOutput = ();
    type Input = Event;
    type Output = crate::Event;
    type Init = ();
    type Root = gtk::Button;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        gtk::Button::with_label("Select Player")
    }

    fn init(_: Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let popover = gtk::Popover::new();

        let listbox = gtk::ListBox::builder()
            .selection_mode(SelectionMode::None)
            .css_classes(vec![String::from("boxed-list")])
            .build();

        let warden = Warden::builder()
            .detach_worker(())
            .forward(sender.input_sender(), identity);

        popover.set_parent(root);
        popover.set_child(Some(&listbox));

        root.connect_clicked(clone!(@strong sender => move |_| {
            sender.input(Event::Button);
        }));

        listbox.connect_row_activated(clone!(@strong sender => move |_, r| {
            sender.input(Event::Row(r.index() as usize));
        }));

        let model = Selector {warden};
        let widgets = Widgets {popover, listbox, rows: Vec::new()};
        ComponentParts {model, widgets}
    }

    fn update_with_view(&mut self, widgets: &mut Self::Widgets, event: Self::Input, sender: ComponentSender<Self>) {
        match event {
            Event::Button => self.warden.emit(warden::Event::Players),
            Event::Row(index) => self.warden.emit(warden::Event::Select(index)),
            Event::Bubble(event) => sender.output(event),

            Event::Failed(err) => {
                let err = err.to_string();
                let row = adw::ActionRow::builder()
                    .title(&err)
                    .build();

                widgets.clear();
                widgets.listbox.append(&row);
                widgets.rows.push(row);
                widgets.popover.show();
            }

            Event::Fetched(endpoints) => {
                widgets.clear();

                for endpoint in endpoints {
                    let row = adw::ActionRow::builder()
                        .title(&endpoint.identity)
                        .subtitle(&endpoint.bus_name)
                        .activatable(true)
                        .build();

                    widgets.listbox.append(&row);
                    widgets.rows.push(row);
                }

                widgets.popover.show();
            }
        }
    }
}
