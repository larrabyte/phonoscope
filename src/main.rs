mod ui;

use gtk4::prelude::*;
use clap::Parser;

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

fn get_players<'a>() -> Vec<mpris::Player<'a>> {
    // TODO: Do we really need to create a new PlayerFinder instance every time?
    let finder = mpris::PlayerFinder::new().expect("D-Bus communication error occurred.");
    finder.find_all().expect("D-Bus communication error occurred.")
}

fn print_player_information(player: &mpris::Player) {
    let identity = player.identity();
    let unique_name = player.unique_name();
    let metadata = player.get_metadata();

    let title = match &metadata {
        Ok(data) => data.title().unwrap_or("No track currently playing."),
        Err(_) => "Unable to ascertain current track."
    };

    println!("Found player: {identity} (bus name {unique_name}) ({title})");
}

fn main() {
    let app = gtk4::Application::builder()
        .application_id("dev.larrabyte.phonoscope")
        .build();

    let args = Arguments::parse();

    if args.list_all_players {
        get_players().iter().for_each(print_player_information);
        return;
    }

    app.connect_activate(move |app| {
        let players = get_players();
        let player = match &args.identity {
            Some(identity) => players.into_iter().find(|p| {identity == p.identity()}).expect("No MPRIS player with specified identity found."),
            None => players.into_iter().next().expect("No active MPRIS players found.")
        };

        print_player_information(&player);

        let (window, title, lyrics) = ui::build_ui(app);
        let anchor = args.source.clone().unwrap_or_else(|| "./".to_string());

        // TODO: Address intermittent metadata failures when rapidly switching tracks.
        glib::timeout_add_seconds_local(1, move || {
            let metadata = player.get_metadata().unwrap();
            let track = metadata.title();

            match track {
                Some(name) if name != title.text().as_str() => {
                    let path = anchor.clone() + "/" + name + ".lyrics";
                    title.set_text(name);

                    match std::fs::read_to_string(&path) {
                        // TODO: Ruby character rendering.
                        // TODO: Synchronised lyric rendering.
                        Ok(text) => lyrics.set_text(&text),
                        Err(err) => lyrics.set_text(format!("{}: {}", err, path).as_ref())
                    }
                },

                None if title.text().as_str() != "No track currently playing." => {
                    title.set_text("No track currently playing.");
                    lyrics.set_text("No lyrics available for display.");
                },

                Some(_) | None => {}
            }

            glib::Continue(true)
        });

        window.show();
    });

    // Prevent GTK from handing any arguments.
    let nothing: Vec<&str> = Vec::new();
    app.run_with_args(&nothing);
}
