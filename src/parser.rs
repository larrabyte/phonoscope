use std::{path::PathBuf, time::Duration, fs};
use crate::display;
use relm::Worker;

/// The lyric format is based off of LRC, with two additions:
/// 
/// 1. There is a whitespace character inserted in between the timestamp and the line.
/// 2. A whitespace-separated word that contains characters specified in Ruby::new()
///    will be split in half by that character, with the left hand side used as the
///    characters for display and the right hand side used as the reading of said characters.

pub struct Parser;

#[derive(Debug)]
pub enum Event {
    Parse(String)
}

#[derive(Debug)]
pub struct Lyric {
    pub timestamp: Duration,
    pub rubies: Vec<Ruby>
}

#[derive(Debug)]
pub struct Ruby {
    pub characters: String,
    pub reading: Option<String>
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Parse
}

impl Lyric {
    pub fn new(line: &str) -> Result<Self, Error> {
        // 1. Split the line over the first whitespace character.
        // 2. Parse the LHS as the timestamp and the RHS as a series of chunks.
        // 3. Process each chunk using Ruby::new().
        // 4. Collect and return (possibly an Error if one arises).
        let mut parts = line.splitn(2, char::is_whitespace);

        let timestamp = {
            let part = parts.next().ok_or(Error::Parse)?;

            let (left, right) = ('[', ']');
            let offset = left.len_utf8();
            let start = part.find(left).ok_or(Error::Parse)?;
            let end = part.find(right).ok_or(Error::Parse)?;
            let numbers = &part[start+offset..end];

            let mut numbers = numbers.split(':');
            let minutes = numbers.next().and_then(|d| d.parse::<u64>().ok()).ok_or(Error::Parse)?;

            let mut numbers = numbers.next().ok_or(Error::Parse)?.split('.');
            let seconds = numbers.next().and_then(|d| d.parse::<u64>().ok()).ok_or(Error::Parse)?;
            let centiseconds = numbers.next().and_then(|d| d.parse::<u32>().ok()).ok_or(Error::Parse)?;

            Duration::new(60 * minutes + seconds, 10_000_000 * centiseconds)
        };

        // 32 words seems like a reasonable number.
        let mut rubies = Vec::with_capacity(32);

        for chunk in parts.next().map(|p| p.split(char::is_whitespace)).ok_or(Error::Parse)? {
            let ruby = Ruby::new(chunk).or(Err(Error::Parse))?;
            rubies.push(ruby);
        }

        Ok(Self {timestamp, rubies})
    }
}

impl Ruby {
    pub fn new(chunk: &str) -> Result<Self, Error> {
        let mut iterator = chunk.split(&['|', '｜'][..]).map(String::from);

        Ok(Self {
            characters: iterator.next().ok_or(Error::Parse)?,
            reading: iterator.next()
        })
    }
}

impl Worker for Parser {
    type Init = ();
    type Input = Event;
    type Output = display::Event;

    fn init(_: Self::Init, _: relm::ComponentSender<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, event: Self::Input, sender: relm::ComponentSender<Self>) {
        match event {
            Event::Parse(path) => {
                let path = String::from("./lyrics/") + &path + ".lrc";
                let path = PathBuf::from(path).canonicalize();
                let data = path.and_then(fs::read_to_string);

                match data {
                    Ok(data) => {
                        let lyrics = data.lines()
                            .filter(|l| !l.is_empty())
                            .map(Lyric::new)
                            .collect::<Vec<Result<Lyric, Error>>>();

                        if let Some(Err(err)) = lyrics.iter().find(|l| l.is_err()) {
                            let message = display::Event::ParseFailed(*err);
                            sender.output(message);
                        }

                        else {
                            let lyrics = lyrics.into_iter().map(|l| l.unwrap()).collect();
                            let message = display::Event::Success(lyrics);
                            sender.output(message);
                        }
                    }

                    Err(err) => {
                        let message = display::Event::IOFailed(err);
                        sender.output(message);
                    }
                }
            }
        }
    }
}
