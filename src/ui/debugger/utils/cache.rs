use crossbeam_channel::Receiver;

use crate::debugger::EmuSnapshot;

pub struct Cache {
    data: EmuSnapshot,
    receiver: Receiver<EmuSnapshot>,
}

impl Cache {
    pub fn new(receiver: Receiver<EmuSnapshot>) -> Self {
        Self {
            data: EmuSnapshot::default(),
            receiver,
        }
    }

    pub fn get(&mut self) -> EmuSnapshot {
        if let Some(data) = self.receiver.try_iter().last() {
            self.data = data;
        }
        self.data.clone()
    }
}
