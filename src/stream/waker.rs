use std::task::Waker;

use futures::task::AtomicWaker;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WakeMode {
    Tx,
    Rx,
}

#[derive(Default, Debug)]
pub struct WakerProxy {
    tx_waker: AtomicWaker,
    rx_waker: AtomicWaker,
}

impl WakerProxy {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register(&self, mode: WakeMode, waker: &Waker) {
        if matches!(mode, WakeMode::Tx) {
            self.tx_waker.register(waker);
        } else {
            self.rx_waker.register(waker);
        }
    }

    pub fn wake(&self, mode: WakeMode) {
        if matches!(mode, WakeMode::Tx) {
            self.tx_waker.wake();
        } else {
            self.rx_waker.wake();
        }
    }
}
