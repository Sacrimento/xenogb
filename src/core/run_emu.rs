use super::cpu::instructions::CPURegisterId;
use super::cpu::{CLOCK_SPEED, LR35902CPU};
use super::io::video::ppu::Vbuf;
use super::io_event::{IOEvent, IOListener};
use super::mem::bus::Bus;
use super::playback::Playback;
use crate::core::io::video::ppu::{RESX, RESY};
use crate::core::utils::{dump_regs, vbuf_snapshot};
use crate::debugger::{Debugger, DebuggerCommand, EmuSnapshot};

use std::backtrace::Backtrace;
use std::panic;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use log::info;

#[derive(Clone, Debug, Copy)]
pub enum StopCondition {
    LDBB,
    TIMER(u32),
}

impl std::str::FromStr for StopCondition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "LDBB" => Ok(Self::LDBB),
            _ if s.to_ascii_uppercase().starts_with("TIMER") => {
                let inner = s.trim_start_matches("TIMER(").trim_end_matches(')');
                inner
                    .parse::<u32>()
                    .map(Self::TIMER)
                    .map_err(|_| "Invalid TIMER value".into())
            }
            _ => Err("Invalid stop condition".into()),
        }
    }
}

pub fn run_headless(
    mut cpu: LR35902CPU,
    video_channel_rc: Receiver<Vbuf>,
    sc: Option<StopCondition>,
) {
    let mut last_frame: Vbuf = [0; RESX * RESY];
    let start = Instant::now();

    loop {
        if let Some(condition) = sc {
            match condition {
                StopCondition::LDBB => {
                    let instr = cpu.current_instruction;
                    if instr.name == "LD"
                        && matches!(instr.reg1, Some(CPURegisterId::B))
                        && matches!(instr.reg2, Some(CPURegisterId::B))
                    {
                        dump_regs(&cpu);
                        return vbuf_snapshot(last_frame);
                    }
                }
                StopCondition::TIMER(secs) => {
                    if start.elapsed() > Duration::from_secs(secs as u64) {
                        dump_regs(&cpu);
                        return vbuf_snapshot(last_frame);
                    }
                }
            }
        }

        cpu.step();

        if let Ok(frame) = video_channel_rc.try_recv() {
            last_frame = frame;
        }
    }
}

fn run(
    cpu: &mut LR35902CPU,
    debugger: &mut Debugger,
    io_listener: IOListener,
    mut playback: Playback,
) {
    loop {
        debugger.handle_events(cpu);

        if debugger.cpu_should_step(cpu) {
            io_listener.handle_events(cpu, &mut playback);

            debugger.executing_pc = cpu.pc();
            cpu.step();
        }

        debugger.collect(cpu);
    }
}

#[derive(Clone)]
pub struct EmuCrash {
    pub reason: String,
    pub backtrace: String,
    pub addr: u16,
}

pub struct EmuState {
    thread: Option<JoinHandle<Result<(), EmuCrash>>>,
    crash: Option<EmuCrash>,
}

impl EmuState {
    pub fn new(thread: JoinHandle<Result<(), EmuCrash>>) -> Self {
        Self {
            thread: Some(thread),
            crash: None,
        }
    }

    pub fn is_dead(&mut self) -> bool {
        if self.crash.is_some() {
            return true;
        }

        if self.thread.as_ref().is_some_and(|t| t.is_finished()) {
            self.crash = Some(
                self.thread
                    .take()
                    .unwrap()
                    .join()
                    .ok()
                    .and_then(|r| r.err())
                    .unwrap(),
            );
            return true;
        }
        false
    }

    pub fn crash_info(&self) -> &EmuCrash {
        self.crash.as_ref().expect("The emulator did not crash...")
    }
}

pub fn run_emu_thread(
    bus: Bus,
    debug: bool,
    serial: bool,
    record_enabled: bool,
    record_path: Option<PathBuf>,
    replay_path: Option<PathBuf>,
) -> (
    EmuState,
    (
        Sender<IOEvent>,
        Sender<DebuggerCommand>,
        Receiver<EmuSnapshot>,
    ),
) {
    let (dbg_cmd_sd, dbg_cmd_rc) = unbounded();
    let (dbg_data_sd, dbg_data_rc) = bounded(1);
    let (io_events_sd, io_events_rc) = unbounded();

    let crash_info = Arc::new(Mutex::new(None));

    let mut cpu = LR35902CPU::new(bus, serial, CLOCK_SPEED);
    let mut dbg = Debugger::new(debug, dbg_cmd_rc, dbg_data_sd);

    let thread = std::thread::spawn(move || {
        panic::set_hook(Box::new({
            let crash_info = crash_info.clone();

            move |info| {
                let reason = info
                    .payload()
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_string())
                    .or_else(|| info.payload().downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| String::from("<unknown reason>"));

                let backtrace = Backtrace::force_capture().to_string();

                *crash_info.lock().unwrap() = Some(EmuCrash {
                    reason,
                    backtrace,
                    addr: 0, // addr is set afterwards by the debugger
                })
            }
        }));

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            info!(
                "Starting emulator with serial:{} debug:{} record:{} replay:{:?}",
                serial, debug, record_enabled, replay_path
            );
            run(
                &mut cpu,
                &mut dbg,
                IOListener::new(io_events_rc),
                Playback::new(record_enabled, record_path, replay_path),
            );
        }));

        match result {
            Ok(_) => Ok(()),
            Err(_) => {
                let mut crash_info =
                    crash_info
                        .lock()
                        .unwrap()
                        .take()
                        .unwrap_or_else(|| EmuCrash {
                            reason: "<unknown reason>".to_string(),
                            backtrace: "<no backtrace>".to_string(),
                            addr: 0,
                        });

                crash_info.addr = dbg.executing_pc;
                dbg.died(&cpu, crash_info.clone());

                Err(crash_info)
            }
        }
    });

    (
        EmuState::new(thread),
        (io_events_sd, dbg_cmd_sd, dbg_data_rc),
    )
}
