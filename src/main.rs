mod players;
mod ruby;
mod ui;

use gtk4::{Application, Label, CssProvider, Orientation, StyleContext};
use glib::{timeout_add_seconds_local, Continue};
use std::cell::RefCell;
use gtk4::gdk::Display;
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

fn clear(lyrics: &gtk4::Box, widgets: &RefCell<Vec<gtk4::Box>>) {
    let mut children = widgets.borrow_mut();
    children.iter().for_each(|w| lyrics.remove(w));
    children.clear();
}

fn main() {
    let args = Arguments::parse();
    let app = Application::builder()
        .application_id("dev.larrabyte.phonoscope")
        .build();

    if args.list_all_players {
        return players::all().iter().for_each(players::print);
    }

    app.connect_startup(|_| {
        // Load default CSS.
        let provider = CssProvider::new();
        let data = include_bytes!("style.css");
        provider.load_from_data(data);

        StyleContext::add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

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
                    let path = anchor.clone() + "/" + name + ".lyrics";
                    clear(&lyrics, &widgets);
                    title.set_text(name);

                    match std::fs::read_to_string(&path) {
                        // TODO: Gracefully reject invalid lyrics.
                        Ok(data) => {
                            rubies.replace(Line::from_filedata(&data));
                        },

                        Err(err) => {
                            let mut children = widgets.borrow_mut();
                            let widget = gtk4::Box::new(Orientation::Horizontal, 0);
                            let error = format!("{}: {}", err, path);
                            let text = Label::new(Some(&error));
                            widget.append(&text);
                            widget.show();

                            lyrics.append(&widget);
                            children.push(widget);
                        }
                    }
                },

                None if title.text().as_str() != "No track currently playing." => {
                    title.set_text("No track currently playing.");
                    clear(&lyrics, &widgets);
                },

                Some(_) | None => {}
            }

            if let Some(line) = players::current_line(&player, &rubies.borrow()) {
                clear(&lyrics, &widgets);
                let mut children = widgets.borrow_mut();

                // Insert the new line's contents.
                for lyric in &line.lyrics {
                    let widget = gtk4::Box::builder()
                        .orientation(Orientation::Vertical)
                        .valign(gtk4::Align::End)
                        .build();

                    let text = Label::builder()
                        .label(&lyric.characters)
                        .css_name("characters")
                        .build();

                    let reading = Label::builder()
                        .label(lyric.reading.as_deref().unwrap_or(""))
                        .css_name("reading")
                        .build();

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
