mod players;
mod ruby;
mod ui;

use glib::{timeout_add_seconds_local, Continue};
use std::cell::RefCell;
use gtk4::Application;
use gtk4::prelude::*;
use clap::Parser;
use ruby::Line;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Arguments {
    /// List all detected players and then exit.
    #[arg(short, long, default_value_t = false)]
    list_all_players: bool,

    /// MPRIS identity of the player to target.
    #[arg(short, long)]
    identity: Option<String>,

    /// Directory where lyrics are stored.
    #[arg(short, long)]
    source: Option<String>
}

fn main() {
    let args = Arguments::parse();
    let app = Application::builder()
        .application_id("dev.larrabyte.phonoscope")
        .build();

    if args.list_all_players {
        return players::all().iter().for_each(players::print);
    }

    app.connect_activate(move |app| {
        let players = players::all();
        let player = match &args.identity {
            Some(identity) => players.into_iter().find(|p| {identity == p.identity()}).expect("No MPRIS player with specified identity found."),
            None => players.into_iter().next().expect("No active MPRIS players found.")
        };

        players::print(&player);

        // Source directory for lyrics. Cloned here to avoid the wrath of the borrow checker.
        let anchor = args.source.clone().unwrap_or_else(|| "./lyrics".to_string());

        // UI elements for the display closure below.
        let (window, title, lyrics) = ui::build_ui(app);
        let widgets: RefCell<Vec<gtk4::Box>> = RefCell::default();
        let rubies: RefCell<Vec<Line>> = RefCell::default();

        // TODO: Address intermittent metadata failures when rapidly switching tracks.
        timeout_add_seconds_local(1, move || {
            let metadata = player.get_metadata().unwrap();
            let track = metadata.title();

            match track {
                Some(name) if name != title.text().as_str() => {
                    title.set_text(name);

                    let path = anchor.clone() + "/" + name + ".lyrics";
                    match std::fs::read_to_string(&path) {
                        // TODO: Gracefully reject invalid lyrics.
                        Ok(data) => {rubies.replace(Line::from_filedata(&data));},
                        Err(err) => {println!("{err:?}");}
                    }
                },

                None if title.text().as_str() != "No track currently playing." => {
                    title.set_text("No track currently playing.");
                },

                Some(_) | None => {}
            }

            if let Some(line) = players::current_line(&player, &rubies.borrow()) {
                // Clear the old line's contents off the display.
                let mut children = widgets.borrow_mut();
                children.iter().for_each(|w| lyrics.remove(w));
                children.clear();

                // Insert the new line's contents.
                for lyric in &line.lyrics {
                    let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                    let text = gtk4::Label::new(Some(&lyric.characters));
                    let reading = gtk4::Label::new(lyric.reading.as_deref());

                    widget.append(&reading);
                    widget.append(&text);
                    widget.show();
                    lyrics.append(&widget);
                    children.push(widget);
                }
            }

            Continue(true)
        });

        window.show();
    });

    // Prevent GTK from handing any arguments.
    let nothing: Vec<&str> = Vec::new();
    app.run_with_args(&nothing);
}
