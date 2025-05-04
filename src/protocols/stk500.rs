use tracing::{debug, info};

use crate::constants::TRANSPORT_THREAD_SLEEP_MICROS;
use crate::error::AvrError;
use crate::interface::common::ProgrammerInterface;
use crate::interface::serialport::SerialPortProgrammer;
use crate::{ProgrammerTrait, error::AvrResult};
use std::sync::{Arc, Mutex, mpsc};

#[repr(u8)]
pub enum Stk500Message {
    CmndStkGetSync = 0x30,
    CmndStkSetDevice = 0x42,
    CmndStkEnterProgMode = 0x50,
    // CmndStkLoadAddress = 0x55,
    // CmndStkProgPage = 0x64,
    // CmndStkLeaveProgMode = 0x51,
    CmndStkReadSign = 0x75,
    SyncCrcEop = 0x20,
    // RespStkNoSync = 0x15,
    RespStkInSync = 0x14,
    RespStkOk = 0x10,
    // CmndStkReadPage = 0x74,
}

pub struct Stk500Params {
    pub port: String,
    pub baud: u32,
    pub signature: Vec<u8>,
}

pub(crate) struct Stk500 {
    source: mpsc::Receiver<Vec<u8>>,
    sink: mpsc::Sender<Vec<u8>>,

    transport: Arc<Mutex<Box<dyn ProgrammerInterface + Send>>>,
    params: Stk500Params,
}

impl Stk500 {
    pub fn new(params: Stk500Params) -> AvrResult<Self> {
        let transport: Box<dyn ProgrammerInterface + Send> =
            Box::new(SerialPortProgrammer::new(params.port.clone(), params.baud)?);
        let (sink, sender_rx) = mpsc::channel();
        let (receiver_tx, source) = mpsc::channel();

        let transport = Arc::new(Mutex::new(transport));
        let transport_sender = Arc::clone(&transport);
        let transport_receiver = Arc::clone(&transport_sender);

        // Sender thread
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_micros(
                    TRANSPORT_THREAD_SLEEP_MICROS,
                ));
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
                std::thread::sleep(std::time::Duration::from_micros(
                    TRANSPORT_THREAD_SLEEP_MICROS,
                ));
                let mut transport = transport_receiver
                    .lock()
                    .expect("Failed to lock transport (receiver thread)");
                match transport.receive() {
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
            params,
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

    fn send_command_and_verify_response(
        &self,
        cmd: Vec<u8>,
        expected_response: Vec<u8>,
    ) -> AvrResult<()> {
        let mut done = false;
        for _ in 0..3 {
            self.send_command(cmd.clone())?;
            let response = self.receive_response()?;

            if response == expected_response {
                done = true;
                break;
            }
        }

        if done {
            Ok(())
        } else {
            Err(AvrError::ProgrammerError(format!(
                "Did not receive expected response {:?} for command {:?}",
                cmd, expected_response
            )))
        }
    }

    pub(crate) fn sync(&self) -> AvrResult<()> {
        self.send_command_and_verify_response(
            vec![
                Stk500Message::CmndStkGetSync as u8,
                Stk500Message::SyncCrcEop as u8,
            ],
            vec![
                Stk500Message::RespStkInSync as u8,
                Stk500Message::RespStkOk as u8,
            ],
        )?;

        debug!("Synced with MCU");
        Ok(())
    }

    fn verify_signature(&self) -> AvrResult<()> {
        self.send_command_and_verify_response(
            vec![
                Stk500Message::CmndStkReadSign as u8,
                Stk500Message::SyncCrcEop as u8,
            ],
            vec![
                vec![Stk500Message::RespStkInSync as u8],
                self.params.signature.clone(),
                vec![Stk500Message::RespStkOk as u8],
            ]
            .concat(),
        )?;

        debug!("Verified board signature");
        Ok(())
    }

    fn set_options(&self) -> AvrResult<()> {
        self.send_command_and_verify_response(
            vec![
                Stk500Message::CmndStkSetDevice as u8,
                0, // Device code
                0, // Revision
                0, // ProgType
                0, // ParMode
                0, // Polling
                0, // SelfTimed
                0, // LockBytes
                0, // FuseBytes
                0, // FlashPollVal1
                0, // FlashPollVal2
                0, // eepromPollVal1
                0, // eepromPollVal2
                0, // PageSizeHigh
                0, // PageSizeLow
                0, // eepromSizeHigh
                0, // eepromSizeLow
                0, // FlashSize4
                0, // FlashSize3
                0, // FlashSize2
                0, // FlashSize1
                Stk500Message::SyncCrcEop as u8,
            ],
            vec![
                Stk500Message::RespStkInSync as u8,
                Stk500Message::RespStkOk as u8,
            ],
        )?;
        debug!("Set options");
        Ok(())
    }

    fn enter_programming_mode(&self) -> AvrResult<()> {
        self.send_command_and_verify_response(
            vec![
                Stk500Message::CmndStkEnterProgMode as u8,
                Stk500Message::SyncCrcEop as u8,
            ],
            vec![
                Stk500Message::RespStkInSync as u8,
                Stk500Message::RespStkOk as u8,
            ],
        )?;

        debug!("Entered programming mode!");
        Ok(())
    }
}

impl ProgrammerTrait for Stk500 {
    fn program_firmware(&self, firmware: Vec<u8>) -> AvrResult<()> {
        let _ = firmware;
        self.reset()?;

        self.sync()?;
        self.sync()?;
        self.sync()?;

        self.verify_signature()?;
        self.set_options()?;
        self.enter_programming_mode()?;

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
