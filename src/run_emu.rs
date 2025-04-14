use crate::cpu::{CLOCK_SPEED, LR35902CPU};
use crate::debugger::{Debugger, DebuggerCommand, EmuSnapshot};
use crate::io::video::ppu::Vbuf;
use crate::io_event::{IOEvent, IOListener};
use crate::mem::bus::Bus;
use crate::Playback;

use std::backtrace::Backtrace;
use std::panic;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use log::info;

#[cfg(unix)]
pub fn run_headless(mut cpu: LR35902CPU, video_channel_rc: Receiver<Vbuf>) {
    use crate::io::video::ppu::{RESX, RESY};
    use crate::utils::vbuf_snapshot;
    use log::info;
    use signal_hook::consts::SIGUSR1;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    let usr1 = Arc::new(AtomicBool::new(false));
    let mut last_frame: Vbuf = [0; RESX * RESY];
    signal_hook::flag::register(SIGUSR1, Arc::clone(&usr1)).unwrap();

    info!("Running in headless mode");

    loop {
        cpu.step();
        if usr1.load(Ordering::Relaxed) {
            vbuf_snapshot(last_frame);
            return;
        }
        if let Ok(frame) = video_channel_rc.try_recv() {
            last_frame = frame;
        }
    }
}

#[cfg(windows)]
pub fn run_headless(mut _cpu: LR35902CPU, _video_channel_rc: Receiver<Vbuf>) {
    panic!("Running headless mode is not yet supported on windows")
}

fn run(
    mut cpu: LR35902CPU,
    mut debugger: Debugger,
    io_listener: IOListener,
    mut playback: Playback,
) {
    loop {
        debugger.handle_events(&mut cpu);

        if debugger.cpu_should_step(&cpu) {
            io_listener.handle_events(&mut cpu, &mut playback);
            cpu.step();
        }

        debugger.collect(&cpu);
    }
}

pub struct EmuCrash {
    pub reason: String,
    pub backtrace: String,
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

                *crash_info.lock().unwrap() = Some(EmuCrash { reason, backtrace })
            }
        }));

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            info!(
                "Starting emulator with serial:{} debug:{} record:{} replay:{:?}",
                serial, debug, record_enabled, replay_path
            );
            run(
                LR35902CPU::new(bus, serial, CLOCK_SPEED),
                Debugger::new(debug, dbg_cmd_rc, dbg_data_sd),
                IOListener::new(io_events_rc),
                Playback::new(record_enabled, record_path, replay_path),
            );
        }));

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(crash_info
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| EmuCrash {
                    reason: "<unknown reason>".to_string(),
                    backtrace: "<no backtrace>".to_string(),
                })),
        }
    });

    (
        EmuState::new(thread),
        (io_events_sd, dbg_cmd_sd, dbg_data_rc),
    )
}
