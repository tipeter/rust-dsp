use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;

#[derive(Debug)]
pub enum DProcCommand {
    SendData(Vec<f64>),
    DoProcessing,
}

#[derive(Debug)]
pub enum DProcResponse {
    Data(Vec<f64>),
    ProcProgress(u8),
}

#[derive(Debug)]
pub struct DProc {
    pub from_dproc_rx: Receiver<DProcResponse>,
    pub to_dproc_tx: Sender<DProcCommand>,
}

impl DProc {
    pub fn new<F: Fn() + Send + 'static>(callback: F) -> Self {
        let (from_dproc_tx, from_dproc_rx) = channel();
        let (to_dproc_tx, to_dproc_rx) = channel();

        DProc {
            from_dproc_rx: from_dproc_rx,
            to_dproc_tx: to_dproc_tx,
        }
    }
}
