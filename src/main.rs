mod ui;

use gtk4::prelude::*;
use clap::Parser;
use gtk4::glib;

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

fn print_player_information(player: &mpris::Player) {
    let identity = player.identity();
    let unique_name = player.unique_name();
    let metadata = player.get_metadata();

    let title = match &metadata {
        Ok(data) => data.title().unwrap_or("No track currently playing."),
        Err(_) => "Unable to ascertain current track."
    };

    println!("{identity} (bus name {unique_name}): {title}");
}

fn main() {
    let args = Arguments::parse();
    let finder = mpris::PlayerFinder::new().expect("D-Bus communication error occurred.");

    if args.list_all_players {
        finder.find_all().expect("D-Bus communication error occurred.").iter().for_each(print_player_information);
    }

    else {
        let app = gtk4::Application::builder()
            .application_id("dev.larrabyte.phonoscope")
            .build();

        app.connect_activate(move |app| {
            let (window, title, lyrics) = ui::build_ui(app);
            let players = finder.find_all().expect("D-Bus communication error occurred.");

            // TODO: Allow application startup even with no active player.
            // TODO: Pick the "currently active" (eg. most recently used) player.
            let player = match &args.identity {
                Some(identity) => players.into_iter().find(|p| {p.identity() == identity}).expect("No MPRIS players with specified identity found."),
                None => players.into_iter().next().expect("No MPRIS players found.") 
            };

            let source_directory = match &args.source {
                Some(path) => std::path::PathBuf::from(path),
                None => panic!("Source lyric directory required!")
            };

            print_player_information(&player);

            let closure = glib::clone!(@weak title, @weak lyrics => @default-panic, move || {
                let metadata = player.get_metadata().unwrap();
                let track = metadata.title();

                // Transitioning from some track -> no track.
                if track.is_none() && title.text().as_str() != "No track currently playing." {
                    title.set_text("No track currently playing.");
                    lyrics.set_text("No lyrics available for display.");
                }

                // Transitioning from no track -> some track or some track -> some track.
                else {
                    let name = track.unwrap();
                    if name != title.text().as_str() {
                        let mut path = source_directory.clone();
                        path.push(name.to_owned() + ".lyrics");
                        title.set_text(name);
    
                        match std::fs::read_to_string(&path) {
                            Ok(text) => lyrics.set_text(&text),
    
                            Err(err) => {
                                let reason = err.to_string();
                                let message = format!("{reason} at {path:?}");
                                lyrics.set_text(&message);
                            }
                        };
                    }
                }

                glib::Continue(true)
            });

            let refresh_interval = std::time::Duration::from_millis(1000);
            glib::timeout_add_local(refresh_interval, closure);
            window.present();
        });

        // Prevent GTK from handling any arguments.
        let arguments: Vec<&str> = Vec::new();
        app.run_with_args(&arguments);
    }
}
