use std::str::FromStr;

use clap::{error::ErrorKind, Error, Parser};
use crossbeam_channel::Sender;

use crate::core::cpu::instructions::CPURegisterId;
use crate::debugger::{DebuggerCommand, DynAddr};

impl FromStr for CPURegisterId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "A" => Ok(Self::A),
            "F" => Ok(Self::F),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            "E" => Ok(Self::E),
            "H" => Ok(Self::H),
            "L" => Ok(Self::L),
            "AF" => Ok(Self::AF),
            "BC" => Ok(Self::BC),
            "DE" => Ok(Self::DE),
            "HL" => Ok(Self::HL),
            "SP" => Ok(Self::SP),
            "PC" => Ok(Self::PC),
            _ => Err(Error::new(ErrorKind::ValueValidation)),
        }
    }
}

impl FromStr for DynAddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(reg) = s.strip_prefix("$") {
            return Ok(Self::new(None, Some(CPURegisterId::from_str(reg)?)));
        }

        if let Some(s) = s.strip_prefix("0x") {
            return Ok(Self::new(
                Some(u16::from_str_radix(s, 16).map_err(|_| Error::new(ErrorKind::InvalidValue))?),
                None,
            ));
        }

        Ok(Self::new(
            Some(u16::from_str_radix(s, 10).map_err(|_| Error::new(ErrorKind::InvalidValue))?),
            None,
        ))
    }
}

struct ReplHistory {
    history: Vec<String>,
    cursor: usize,
}

impl ReplHistory {
    pub fn new() -> Self {
        Self {
            history: vec![],
            cursor: 0,
        }
    }

    pub fn push(&mut self, cmd: String) {
        if self.history.last().is_none_or(|c| *c != cmd) {
            self.history.push(cmd);
            self.cursor = self.history.len() - 1;
        }
    }

    pub fn next(&mut self) -> String {
        if self.history.is_empty() {
            return String::new();
        }

        let s = self.history[self.cursor].clone();

        if self.cursor > 0 {
            self.cursor -= 1;
        }

        s
    }

    pub fn prev(&mut self) -> String {
        if self.history.is_empty() {
            return String::new();
        }

        let s = self.history[self.cursor].clone();

        if self.cursor < self.history.len() - 1 {
            self.cursor += 1;
        }

        s
    }
}

#[derive(Parser, Debug)]
#[clap(no_binary_name = true)]
enum ReplCommand {
    Continue,
    Run,
    Step,
    Breakpoint { addr: DynAddr },
}

pub struct Repl {
    sender: Sender<DebuggerCommand>,

    pub cmd: String,
    history: ReplHistory,
}

impl Repl {
    pub fn new(sender: Sender<DebuggerCommand>) -> Self {
        Self {
            sender,
            history: ReplHistory::new(),
            cmd: String::new(),
        }
    }

    pub fn exec(&mut self) -> Result<String, Error> {
        match ReplCommand::try_parse_from(self.cmd.trim().split_whitespace())? {
            ReplCommand::Run | ReplCommand::Continue => {
                self.sender.send(DebuggerCommand::CONTINUE).unwrap()
            }
            ReplCommand::Step => self.sender.send(DebuggerCommand::STEP).unwrap(),
            ReplCommand::Breakpoint { addr } => {
                self.sender.send(DebuggerCommand::BREAKPOINT(addr)).unwrap();
            }
        }

        self.history.push(self.cmd.clone());

        self.cmd.clear();

        Ok(self.history.history.last().unwrap().clone())
    }

    pub fn history_next(&mut self) {
        self.cmd = self.history.next();
    }

    pub fn history_prev(&mut self) {
        self.cmd = self.history.prev();
    }
}
