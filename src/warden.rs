use mpris::{PlayerFinder, Player, Metadata};
use std::time::Duration;
use relm::Worker;

use crate::selector;

use relm::prelude::*;

#[derive(Debug)]
pub struct Warden {
    finder: PlayerFinder,
    players: Vec<Player>,

    // Stores the index of the active player + cached metadata.
    active: Option<(usize, Option<Metadata>)>,
}

// SAFETY: The Rc<PooledConnection> present in the player finder is only ever accessed from this thread.
unsafe impl Send for Warden {}

#[derive(Debug)]
pub enum Event {
    Players,
    Select(usize),
    Poll
}

#[derive(Debug)]
pub struct Endpoint {
    pub bus_name: String,
    pub unique_name: String,
    pub identity: String,
}

impl Endpoint {
    pub fn new(player: &Player) -> Self {
        Self {
            bus_name: player.bus_name().to_owned(),
            unique_name: player.unique_name().to_owned(),
            identity: player.identity().to_owned()
        }
    }
}

impl Worker for Warden {
    type Init = ();
    type Input = Event;
    type Output = selector::Event;

    fn init(_: Self::Init, _: ComponentSender<Self>) -> Self {
        let finder = match PlayerFinder::new() {
            Ok(finder) => finder,
            Err(err) => panic!("{}", err)
        };

        Self {finder, players: Vec::new(), active: None}
    }

    fn update(&mut self, event: Self::Input, sender: ComponentSender<Self>) {
        match event {
            Event::Players => {
                match self.finder.find_all() {
                    Ok(players) => {
                        self.active = None;
                        self.players = players;

                        let payload = self.players.iter().map(Endpoint::new).collect();
                        let message = selector::Event::Fetched(payload);
                        sender.output(message);
                    }

                    Err(err) => {
                        let message = selector::Event::Failed(err);
                        sender.output(message);
                    }
                }
            }

            Event::Select(index) => {
                self.active = Some((index, None));
                sender.input(Event::Poll);
            }

            Event::Poll => {
                let active = self.active.as_ref().map(|(i, m)| (&self.players[*i], m.as_ref()));
                let metadata = active.and_then(|t| t.0.get_metadata().ok());

                if let (Some((_, cache)), Some(metadata)) = (active, metadata) {
                    if cache.is_none() || cache.unwrap().title() != metadata.title() {
                        let title = metadata.title().map(|t| t.to_owned());
                        let payload = crate::Event::Track(title);
                        let message = selector::Event::Bubble(payload);
                        sender.output(message);
                    }

                    if let Ok(timestamp) = active.unwrap().0.get_position() {
                        let payload = crate::Event::Poll(timestamp);
                        let message = selector::Event::Bubble(payload);
                        sender.output(message);
                    }

                    self.active = self.active.as_ref().map(|t| (t.0, Some(metadata)));

                    // TODO: Address intermittent metadata failures.
                    let interval = Duration::from_millis(100);
                    std::thread::sleep(interval);
                    sender.input(Event::Poll);
                }

                else {
                    self.active = None;
                    let payload = crate::Event::Reset;
                    let message = selector::Event::Bubble(payload);
                    sender.output(message);
                }
            }
        }
    }
}
