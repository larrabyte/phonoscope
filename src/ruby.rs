use core::time::Duration;

#[derive(Debug, Clone)]
pub struct Line {
    // TODO: Duration type as a timestamp?
    pub timestamp: Duration,
    pub lyrics: Vec<Ruby>
}

#[derive(Debug, Clone)]
pub struct Ruby {
    pub characters: String,
    pub reading: Option<String>
}

impl Ruby {
    pub fn from_formatted(chunk: &str) -> Self {
        // Expected format: [whitespace]<characters>｜<reading><whitespace>
        let mut cursor = chunk.split('｜');

        Ruby {
            characters: cursor.next().unwrap().to_owned(),
            reading: cursor.next().map(|x| x.to_owned())
        }
    }
}

impl Line {
    pub fn from_formatted(line: &str) -> Self {
        // (minutes):(seconds):(milliseconds) is guaranteed to take up (2+2+3=9) characters on a valid line.
        let integers = line[0..8].split(':').map(|e| e.parse::<u32>().unwrap()).collect::<Vec<u32>>();
        let seconds = integers[0] as u64 * 60 + integers[1] as u64;
        let milliseconds = integers[2] * 1000000;

        Line {
            timestamp: Duration::new(seconds, milliseconds),
            lyrics: line[10..].split_whitespace().map(Ruby::from_formatted).collect()
        }
    }

    pub fn from_filedata(data: &str) -> Vec<Self> {
        data.lines().filter(|l| !l.is_empty()).map(Line::from_formatted).collect()
    }
}
