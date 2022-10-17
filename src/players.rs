use mpris::{PlayerFinder, Player};
use crate::Line;

pub fn all<'a>() -> Vec<Player<'a>> {
    // TODO: Do we really need to create a new PlayerFinder instance every time?
    let finder = PlayerFinder::new().expect("D-Bus communication error occurred.");
    finder.find_all().expect("D-Bus communication error occurred.")
}

pub fn print(player: &Player) {
    let identity = player.identity();
    let unique_name = player.unique_name();
    let metadata = player.get_metadata();

    let title = match &metadata {
        Ok(data) => data.title().unwrap_or("No track currently playing."),
        Err(_) => "Unable to ascertain current track."
    };

    println!("Found player: {identity} (bus name {unique_name}) ({title})");
}

pub fn current_line<'a>(player: &Player, lines: &'a [Line]) -> Option<&'a Line> {
    let cursor = player.get_position().unwrap();
    lines.iter().filter(|l| l.timestamp < cursor).last()
}
