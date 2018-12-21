use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

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

#[derive(Debug)]
pub enum GeneralError {
    SendError(DProcCommand),
}

impl DProc {
    pub fn new<F: Fn() + Send + 'static>(callback: F) -> Self {
        let (from_dproc_tx, from_dproc_rx) = channel();
        let (to_dproc_tx, to_dproc_rx) = channel();

        thread::spawn(move || loop {
            match to_dproc_rx.try_recv() {
                Ok(DProcCommand::SendData(data)) => {
                    println!("Command received: {:?}", data);
                }
                Ok(DProcCommand::DoProcessing) => {}
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    println!("Error: receiver disconnected!");
                }
            }
            thread::sleep(Duration::from_millis(10u64));
        });

        DProc {
            from_dproc_rx: from_dproc_rx,
            to_dproc_tx: to_dproc_tx,
        }
    }

    pub fn send_data_cmd(&self, data: &Vec<f64>) -> Result<(), GeneralError> {
        let tx = &self.to_dproc_tx;
        tx.send(DProcCommand::SendData(data.clone()))
            .map_err(|e| GeneralError::SendError(e.0))?;
        Ok(())
    }
}
