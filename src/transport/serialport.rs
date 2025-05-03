use super::common::ProgrammerInterface;
use crate::constants::{MAX_RESPONSE_SIZE, SERIAL_TIMEOUT_MS};

use crate::error::{AvrError, AvrResult};
use std::io::{Read, Write};

pub type ComPort = String;
pub type BaudRate = u32;
/// Serial port transport layer
pub(crate) struct SerialPortTransport {
    pub serial_port: Box<dyn serialport::SerialPort>,
}

impl SerialPortTransport {
    pub fn new(port: ComPort, baud: BaudRate) -> AvrResult<SerialPortTransport> {
        let serial_port = serialport::new(port, baud)
            .timeout(std::time::Duration::from_millis(SERIAL_TIMEOUT_MS))
            .dtr_on_open(false)
            .open()
            .map_err(|e| AvrError::Communication(format!("{:?}", e)))?;

        Ok(SerialPortTransport { serial_port })
    }
}

impl ProgrammerInterface for SerialPortTransport {
    fn send(&mut self, command: Vec<u8>) -> AvrResult<()> {
        self.serial_port
            .write_all(&command)
            .map_err(|e| AvrError::Communication(format!("{:?}", e)))?;
        Ok(())
    }

    fn receive(&mut self, expected_bytes: usize) -> AvrResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        // Block and keep reading until we have the expected number of bytes
        while buffer.len() < expected_bytes {
            let mut temp_buffer = vec![0; MAX_RESPONSE_SIZE];
            let bytes_read = self
                .serial_port
                .read(&mut temp_buffer)
                // Timeout error is fine, just continue
                .or_else(|e| {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        Ok(0)
                    } else {
                        Err(e)
                    }
                })
                .map_err(|e| AvrError::Communication(format!("{:?}", e)))?;

            if bytes_read == 0 {
                break;
            }

            buffer.extend_from_slice(&temp_buffer[..bytes_read]);
        }

        Ok(buffer)
    }

    fn reset(&mut self) -> AvrResult<()> {
        // Reset logic for the serial port
        self.serial_port
            .write_request_to_send(false)
            .map_err(|e| AvrError::Communication(format!("Failed to set RTS false: {:?}", e)))?;
        self.serial_port
            .write_data_terminal_ready(false)
            .map_err(|e| AvrError::Communication(format!("Failed to set DTR false: {:?}", e)))?;

        std::thread::sleep(std::time::Duration::from_millis(50));

        self.serial_port
            .write_request_to_send(true)
            .map_err(|e| AvrError::Communication(format!("Failed to set RTS true: {:?}", e)))?;
        self.serial_port
            .write_data_terminal_ready(true)
            .map_err(|e| AvrError::Communication(format!("Failed to set DTR true: {:?}", e)))?;
        Ok(())
    }
}
