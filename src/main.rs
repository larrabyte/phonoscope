use gtk4::prelude::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Arguments {
    /// List all detected players and then exit.
    #[arg(short, long, default_value_t = false)]
    list_all_players: bool,

    /// Name of the player to target.
    #[arg(short, long)]
    player: Option<String>
}

fn main() {
    let args = Arguments::parse();
    let finder = mpris::PlayerFinder::new().expect("D-Bus communication error occurred.");

    if args.list_all_players {
        for player in finder.find_all().expect("D-Bus communication error occurred.") {
            let identity = player.identity();
            let unique_name = player.unique_name();
            let metadata = player.get_metadata();

            let title = match &metadata {
                Ok(data) => data.title().unwrap_or("No track currently playing."),
                Err(_) => "Unable to ascertain current track."
            };

            println!("{identity} (bus name {unique_name}): {title}");
        }

        return;
    }

    let app = gtk4::Application::builder()
        .application_id("dev.larrabyte.phonoscope")
        .build();

    app.connect_activate(move |app| {
        let players = finder.find_all().expect("D-Bus communication error occurred.");

        // TODO: Allow application startup even with no active player.
        let player = match &args.player {
            Some(identity) => players.into_iter().find(|p| {p.identity() == identity}).expect("No MPRIS player found with specified identity."),
            None => players.into_iter().next().expect("No active MPRIS player found.")
        };

        let identity = player.identity();
        let unique_name = player.unique_name();
        println!("Using player: {identity} (bus name {unique_name})");

        let title = gtk4::Label::builder()
            .label("No song currently playing.")
            .selectable(true)
            .single_line_mode(true)
            .wrap(false)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .default_width(128)
            .default_height(72)
            .title("Phonoscope")
            .child(&title)
            .build();

        // TODO: Fix intermittent failure to get track metadata when rapidly switching tracks.
        let refresh_interval = std::time::Duration::from_millis(100);
        glib::timeout_add_local(refresh_interval, move || {
            let metadata = player.get_metadata().unwrap();
            let name = metadata.title().unwrap_or("No song currently playing.");
            title.set_text(name);
            glib::Continue(true)
        });

        window.show();
    });

    // Let clap handle argument parsing before the GUI runs.
    let arguments = vec![""];
    app.run_with_args(&arguments);
}
