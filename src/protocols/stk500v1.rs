use tracing::{debug, info, trace};

use crate::constants::TRANSPORT_THREAD_SLEEP_MICROS;
use crate::error::AvrError;
use crate::interface::DeviceInterface;
use crate::interface::serialport::SerialPortDevice;
use crate::{ProgrammerTrait, error::AvrResult};
use std::ptr::read;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

#[repr(u8)]
pub enum Stk500v1Message {
    CmndStkGetSync = 0x30,
    CmndStkSetDevice = 0x42,
    CmndStkEnterProgMode = 0x50,
    CmndStkLoadAddress = 0x55,
    CmndStkProgPage = 0x64,
    CmndStkLeaveProgMode = 0x51,
    CmndStkReadSign = 0x75,
    SyncCrcEop = 0x20,
    // RespStkNoSync = 0x15,
    RespStkInSync = 0x14,
    RespStkOk = 0x10,
    CmndStkReadPage = 0x74,
}

pub struct Stk500v1Params {
    pub port: String,
    pub baud: u32,
    pub signature: Vec<u8>,
    pub page_size: u16,
    pub num_pages: u16,
    pub product_id: Vec<u16>,
    pub verify: bool,
}

pub(crate) struct Stk500 {
    source: mpsc::Receiver<Vec<u8>>,
    sink: mpsc::Sender<Vec<u8>>,

    device_interface: Arc<Mutex<Box<dyn DeviceInterface + Send>>>,
    params: Stk500v1Params,
}

impl Stk500 {
    pub fn new(params: Stk500v1Params) -> AvrResult<Self> {
        let device_interface: Box<dyn DeviceInterface + Send> =
            Box::new(SerialPortDevice::new(params.port.clone(), params.baud)?);
        let (sink, sender_rx) = mpsc::channel();
        let (receiver_tx, source) = mpsc::channel();

        let device_interface = Arc::new(Mutex::new(device_interface));
        let transport_sender = Arc::clone(&device_interface);
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
                        let mut device_interface = transport_sender
                            .lock()
                            .expect("Failed to lock device_interface (sender thread)");
                        if let Err(e) = device_interface.send(command) {
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
                let mut device_interface = transport_receiver
                    .lock()
                    .expect("Failed to lock device_interface (receiver thread)");
                match device_interface.receive() {
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
            device_interface,
            params,
        })
    }

    pub(crate) fn send_command(&self, command: Vec<u8>) -> AvrResult<()> {
        println!("Sent bytes {:?}", command);
        self.sink
            .send(command)
            .map_err(|e| AvrError::Communication(format!("Failed to send command: {:?}", e)))?;
        Ok(())
    }

    pub fn flush_device_buffers(&self) -> AvrResult<()> {
        self.device_interface
            .lock()
            .expect("Could not lock device interface")
            .flush_buffers()?;

        Ok(())
    }

    pub(crate) fn receive_response(&self) -> AvrResult<Vec<u8>> {
        self.source
            .recv()
            .map_err(|e| AvrError::Communication(format!("Failed to receive response: {:?}", e)))
    }

    pub(crate) fn receive_response_with_size(&self, expected_size: usize) -> AvrResult<Vec<u8>> {
        let mut received = Vec::new();

        while received.len() < expected_size {
            let fresh_bytes = self.source.recv().map_err(|e| {
                AvrError::Communication(format!("Failed to receive response: {:?}", e))
            })?;
            received.extend(fresh_bytes);
        }

        info!("Received with size {:?}", received);
        Ok(received)
    }

    fn send_command_and_verify_response(
        &self,
        cmd: Vec<u8>,
        expected_response: Vec<u8>,
    ) -> AvrResult<()> {
        // thread::sleep(std::time::Duration::from_millis(10));
        self.send_command(cmd.clone())?;
        thread::sleep(std::time::Duration::from_millis(4));
        let response = self.receive_response_with_size(expected_response.len())?;

        if response == expected_response {
            Ok(())
        } else {
            Err(AvrError::ProgrammerError(format!(
                "Did not receive expected response {:?} for command {:?}",
                expected_response, cmd
            )))
        }
    }

    fn _send_command_and_verify_response_with_retries(
        &self,
        cmd: Vec<u8>,
        expected_response: Vec<u8>,
    ) -> AvrResult<()> {
        let mut done = false;
        for _ in 0..3 {
            self.send_command(cmd.clone())?;
            thread::sleep(std::time::Duration::from_millis(4));
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
        info!("Attempting to sync with target");
        self.send_command_and_verify_response(
            vec![
                Stk500v1Message::CmndStkGetSync as u8,
                Stk500v1Message::SyncCrcEop as u8,
            ],
            vec![
                Stk500v1Message::RespStkInSync as u8,
                Stk500v1Message::RespStkOk as u8,
            ],
        )?;

        info!("Synced with MCU");
        Ok(())
    }

    fn verify_signature(&self) -> AvrResult<()> {
        self.send_command_and_verify_response(
            vec![
                Stk500v1Message::CmndStkReadSign as u8,
                Stk500v1Message::SyncCrcEop as u8,
            ],
            vec![
                vec![Stk500v1Message::RespStkInSync as u8],
                self.params.signature.clone(),
                vec![Stk500v1Message::RespStkOk as u8],
            ]
            .concat(),
        )?;

        info!("Verified board signature");
        Ok(())
    }

    fn set_options(&self) -> AvrResult<()> {
        self.send_command_and_verify_response(
            vec![
                Stk500v1Message::CmndStkSetDevice as u8,
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
                Stk500v1Message::SyncCrcEop as u8,
            ],
            vec![
                Stk500v1Message::RespStkInSync as u8,
                Stk500v1Message::RespStkOk as u8,
            ],
        )?;
        info!("Set options");
        Ok(())
    }

    fn enter_programming_mode(&self) -> AvrResult<()> {
        self.send_command_and_verify_response(
            vec![
                Stk500v1Message::CmndStkEnterProgMode as u8,
                Stk500v1Message::SyncCrcEop as u8,
            ],
            vec![
                Stk500v1Message::RespStkInSync as u8,
                Stk500v1Message::RespStkOk as u8,
            ],
        )?;

        info!("Entered programming mode!");
        Ok(())
    }

    fn load_address(&self, use_addr: u16) -> AvrResult<()> {
        let high_addr: u8 = ((use_addr >> 8) & 0xFF) as u8;
        let low_addr: u8 = (use_addr & 0xFF) as u8;

        self.send_command_and_verify_response(
            vec![
                Stk500v1Message::CmndStkLoadAddress as u8,
                low_addr,
                high_addr,
                Stk500v1Message::SyncCrcEop as u8,
            ],
            vec![
                Stk500v1Message::RespStkInSync as u8,
                Stk500v1Message::RespStkOk as u8,
            ],
        )?;

        Ok(())
    }

    fn load_page(&self, write_bytes: &[u8]) -> AvrResult<()> {
        let data_len = write_bytes.len() as u16;
        let bytes_high = ((data_len >> 8) & 0xFF) as u8;
        let bytes_low = (data_len & 0xFF) as u8;

        self.send_command_and_verify_response(
            vec![
                vec![
                    Stk500v1Message::CmndStkProgPage as u8,
                    bytes_high,
                    bytes_low,
                    0x46,
                ],
                write_bytes.to_vec(),
                vec![Stk500v1Message::SyncCrcEop as u8],
            ]
            .concat(),
            vec![
                Stk500v1Message::RespStkInSync as u8,
                Stk500v1Message::RespStkOk as u8,
            ],
        )?;

        Ok(())
    }

    fn verify_page(&self, verify_bytes: &[u8]) -> AvrResult<()> {
        let data_len = verify_bytes.len() as u16;
        let size = if data_len > self.params.page_size {
            self.params.page_size
        } else {
            data_len
        };

        let byte_high = ((size >> 8) & 0xFF) as u8;
        let byte_low = (size & 0xFF) as u8;

        self.send_command_and_verify_response(
            vec![
                Stk500v1Message::CmndStkReadPage as u8,
                byte_high,
                byte_low,
                0x46,
                Stk500v1Message::SyncCrcEop as u8,
            ],
            vec![
                vec![Stk500v1Message::RespStkInSync as u8],
                verify_bytes.to_vec(),
                vec![Stk500v1Message::RespStkOk as u8],
            ]
            .concat(),
        )?;

        Ok(())
    }

    fn exit_programming_mode(&self) -> AvrResult<()> {
        self.send_command_and_verify_response(
            vec![
                Stk500v1Message::CmndStkLeaveProgMode as u8,
                Stk500v1Message::SyncCrcEop as u8,
            ],
            vec![
                Stk500v1Message::RespStkInSync as u8,
                Stk500v1Message::RespStkOk as u8,
            ],
        )?;
        Ok(())
    }

    fn upload(&self, bin: Vec<u8>) -> AvrResult<()> {
        info!("Started programming");
        let page_size = self.params.page_size;
        let mut page_addr: u16 = 0;
        let mut use_addr: u16;

        while page_addr < bin.len() as u16 {
            use_addr = page_addr >> 1;

            self.load_address(use_addr)?;
            let end = if bin.len() as u16 > (page_addr + page_size) {
                page_addr + page_size
            } else {
                bin.len() as u16 - 1
            };
            info!("Page addr: {}", page_addr);
            info!("end {}", end);

            let slice = &bin[(page_addr as usize)..(end as usize)];

            info!("Slice len: {}", slice.len());
            if slice.len() == 0 {
                // We've reached the end
                std::thread::sleep(std::time::Duration::from_millis(50));
                break;
            }

            self.load_page(slice)?;
            page_addr += slice.len() as u16;

            std::thread::sleep(std::time::Duration::from_millis(4));
        }

        Ok(())
    }

    fn verify(&self, bin: Vec<u8>) -> AvrResult<()> {
        info!("Started verifying");
        let mut page_addr = 0;
        let mut use_addr;
        let mut read_bytes: &[u8];

        while page_addr < bin.len() {
            use_addr = (page_addr >> 1) as u16;
            self.load_address(use_addr)?;

            read_bytes = &bin[page_addr..if bin.len() > self.params.page_size as usize {
                page_addr as usize + self.params.page_size as usize
            } else {
                bin.len() - 1
            }];

            if read_bytes.len() == 0 {
                break;
            }

            self.verify_page(read_bytes)?;

            page_addr += read_bytes.len();
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
        Ok(())
    }
}

impl ProgrammerTrait for Stk500 {
    fn program_firmware(&self, firmware: Vec<u8>) -> AvrResult<()> {
        let _ = firmware;
        self.reset()?;

        self.sync()?;

        self.verify_signature()?;
        self.set_options()?;
        self.enter_programming_mode()?;

        self.upload(firmware.clone())?;
        // self.verify(firmware)?;

        self.exit_programming_mode()?;

        Ok(())
    }

    fn reset(&self) -> AvrResult<()> {
        self.device_interface
            .lock()
            .map_err(|_| AvrError::Communication("Failed to lock device_interface".to_string()))?
            .reset()
            .map_err(|e| AvrError::Communication(format!("Failed to reset: {:?}", e)))?;
        Ok(())
    }
}
