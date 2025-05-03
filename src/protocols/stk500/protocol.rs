use tracing::info;

use crate::constants::{MIN_RESPONSE_SIZE, TRANSPORT_THREAD_SLEEP_MS};
use crate::error::AvrError;
use crate::transport::common::TransportLayer;
use crate::transport::serialport::SerialPortTransport;
use crate::{ProgrammerTrait, error::AvrResult};
use std::sync::{Arc, Mutex, mpsc};

pub struct Stk500 {
    source: mpsc::Receiver<Vec<u8>>,
    sink: mpsc::Sender<Vec<u8>>,
}

impl Stk500 {
    pub fn new(port: String, baud_rate: u32) -> AvrResult<Self> {
        let transport: Box<dyn TransportLayer + Send> =
            Box::new(SerialPortTransport::new(port, baud_rate)?);
        let (sink, sender_rx) = mpsc::channel();
        let (receiver_tx, source) = mpsc::channel();

        let transport_sender = Arc::new(Mutex::new(transport));
        let transport_receiver = Arc::clone(&transport_sender);

        // Sender thread
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(TRANSPORT_THREAD_SLEEP_MS));
                let recv_result = sender_rx.recv();
                match recv_result {
                    Ok(command) => {
                        let mut transport = transport_sender
                            .lock()
                            .expect("Failed to lock transport (sender thread)");
                        if let Err(e) = transport.send(command) {
                            eprintln!("Error sending command: {:?}", e);
                        }
                    }
                    Err(_) => {
                        info!("Sender thread terminated.");
                        break;
                    }
                }
            }
        });

        // Receiver thread
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(TRANSPORT_THREAD_SLEEP_MS));
                let mut transport = transport_receiver
                    .lock()
                    .expect("Failed to lock transport (receiver thread)");
                match transport.receive(MIN_RESPONSE_SIZE) {
                    Ok(response) => {
                        if let Err(e) = receiver_tx.send(response) {
                            eprintln!("Error sending response: {:?}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving response: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(Stk500 { source, sink })
    }

    pub(crate) fn send_command(&self, command: Vec<u8>) -> AvrResult<()> {
        self.sink
            .send(command)
            .map_err(|e| AvrError::Communication(format!("Failed to send command: {:?}", e)))?;
        Ok(())
    }

    pub(crate) fn receive_response(&self) -> AvrResult<Vec<u8>> {
        self.source
            .recv()
            .map_err(|e| AvrError::Communication(format!("Failed to receive response: {:?}", e)))
    }
}

impl ProgrammerTrait for Stk500 {
    fn program_firmware(&self, firmware: Vec<u8>) -> AvrResult<()> {
        let _ = firmware;

        let command = vec![0xAC, 0x53, 0x00, 0x00]; // Example command
        self.send_command(command)?;
        let _ = self.receive_response()?;

        Ok(())
    }
}
