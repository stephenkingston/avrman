use tracing::info;

use crate::constants::{MIN_RESPONSE_SIZE, TRANSPORT_THREAD_SLEEP_MS};
use crate::error::AvrError;
use crate::transport::common::ProgrammerInterface;
use crate::transport::serialport::SerialPortTransport;
use crate::{ProgrammerTrait, error::AvrResult};
use std::sync::{Arc, Mutex, mpsc};

pub struct Stk500 {
    source: mpsc::Receiver<Vec<u8>>,
    sink: mpsc::Sender<Vec<u8>>,

    transport: Arc<Mutex<Box<dyn ProgrammerInterface + Send>>>,
}

impl Stk500 {
    pub fn new(port: String, baud_rate: u32) -> AvrResult<Self> {
        let transport: Box<dyn ProgrammerInterface + Send> =
            Box::new(SerialPortTransport::new(port, baud_rate)?);
        let (sink, sender_rx) = mpsc::channel();
        let (receiver_tx, source) = mpsc::channel();

        let transport = Arc::new(Mutex::new(transport));
        let transport_sender = Arc::clone(&transport);
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

        Ok(Stk500 {
            source,
            sink,
            transport,
        })
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

    pub(crate) fn sync(&self) -> AvrResult<()> {
        let sync_command = vec![0x30, 0x20];
        let expected_response = vec![0x14, 0x10];
        let mut synced = false;

        for i in 0..15 {
            self.send_command(sync_command.clone())?;
            let response = self.receive_response()?;
            println!("Response {:?}", response);

            if response == expected_response {
                println!("Synchronized with STK500.");
                synced = true;
                break;
            } else {
                println!("Failed to synchronize with STK500. Attempt {}/3", i + 1);
            }
        }

        if synced {
            Ok(())
        } else {
            Err(AvrError::ProgrammerError(format!(
                "Stk500: Failed to sync with target"
            )))
        }
    }

    fn verify_signature(&self) -> AvrResult<()> {
        let cmd = vec![0x75, 0x20];
        let expected_response = vec![0x14, 0x1e, 0x95, 0x0f, 0x10];
        let mut verified = false;

        for _ in 0..30 {
            self.send_command(cmd.clone())?;
            let resp = self.receive_response()?;

            println!("signature response {:?}", resp);

            if expected_response == resp {
                println!("Verified signature!");
                verified = true;
                break;
            }
        }

        if verified {
            Ok(())
        } else {
            Err(AvrError::ProgrammerError(format!(
                "Could not verify device signature"
            )))
        }
    }
}

impl ProgrammerTrait for Stk500 {
    fn program_firmware(&self, firmware: Vec<u8>) -> AvrResult<()> {
        let _ = firmware;
        self.reset()?;

        self.sync()?;
        self.verify_signature()?;

        Ok(())
    }

    fn reset(&self) -> AvrResult<()> {
        self.transport
            .lock()
            .map_err(|_| AvrError::Communication("Failed to lock transport".to_string()))?
            .reset()
            .map_err(|e| AvrError::Communication(format!("Failed to reset: {:?}", e)))?;
        Ok(())
    }
}
