use crate::io_event::IOEvent;
use std::fs::File;
use std::io::BufReader;
use std::io::{BufRead, Write};
use std::path::PathBuf;

pub struct Recorder {
    file: Option<File>,
}

impl Recorder {
    pub fn new(record_enabled: bool, record_path: Option<PathBuf>) -> Self {
        let file = if record_enabled || record_path.is_some() {
            record_path
                .or_else(|| {
                    Some(PathBuf::from(format!(
                        "record_{}.xenogb",
                        chrono::Local::now().timestamp()
                    )))
                })
                .map(|p| File::create(p).expect("Could not open record_file"))
        } else {
            None
        };

        Self { file }
    }

    pub fn enabled(&self) -> bool {
        self.file.is_some()
    }

    pub fn record(&mut self, event: &IOEvent, frame: u64, tick: u32) {
        if let Some(file) = &mut self.file {
            if !matches!(event, &IOEvent::CLOSE) {
                file.write(format!("{frame} {tick} {event}\n").as_bytes())
                    .expect("Could not record input");
            }
        }
    }
}

pub struct EventIter<'a> {
    player: &'a mut Player,
    target_frame: u64,
    target_tick: u32,
}

impl<'a> Iterator for EventIter<'a> {
    type Item = IOEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.player.enabled()
            && self.player.cur_frame <= self.target_frame
            && self.player.cur_tick <= self.target_tick
        {
            let evt = self.player.cur_event;
            self.player.next();
            Some(evt)
        } else {
            None
        }
    }
}

pub struct Player {
    file: Option<BufReader<File>>,

    line_buffer: String,
    cur_frame: u64,
    cur_tick: u32,
    cur_event: IOEvent,
}

impl Player {
    pub fn new(replay_path: Option<PathBuf>) -> Self {
        let file =
            replay_path.map(|p| BufReader::new(File::open(p).expect("Could not open replay_file")));

        let mut p = Self {
            file,
            line_buffer: String::new(),
            cur_event: IOEvent::CLOSE,
            cur_frame: 0,
            cur_tick: 0,
        };

        p.next();
        p
    }

    pub fn enabled(&self) -> bool {
        self.file.is_some() && !self.line_buffer.is_empty()
    }

    pub fn events(&mut self, frames: u64, ticks: u32) -> EventIter<'_> {
        EventIter {
            player: self,
            target_frame: frames,
            target_tick: ticks,
        }
    }

    fn next(&mut self) {
        if let Some(f) = &mut self.file {
            self.line_buffer.clear();
            f.read_line(&mut self.line_buffer)
                .expect("Could not read line in replay file");

            if self.line_buffer.is_empty() {
                println!("Replay finished!");
                return;
            }

            let mut split = self.line_buffer.split_whitespace();
            self.cur_frame = split
                .next()
                .expect("Invalid frame in replay file")
                .parse()
                .expect("Invalid frame in replay file");
            self.cur_tick = split
                .next()
                .expect("Invalid tick in replay file")
                .parse()
                .expect("Invalid tick in replay file");
            let action = split.next().expect("Invalid command in replay file");

            self.cur_event =
                IOEvent::from_strs(action, split.next()).expect("Invalid command in replay file");
        }
    }
}

pub struct Playback {
    pub recorder: Recorder,
    pub player: Player,
}

impl Playback {
    pub fn new(
        record_enabled: bool,
        record_path: Option<PathBuf>,
        replay_path: Option<PathBuf>,
    ) -> Self {
        Self {
            recorder: Recorder::new(record_enabled, record_path),
            player: Player::new(replay_path),
        }
    }
}
