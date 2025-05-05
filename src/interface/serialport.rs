use super::DeviceInterface;
use crate::constants::{
    MAX_RESPONSE_SIZE, POST_RESET_BOOTUP_DELAY_MS, RESET_DTR_RTS_LOW_MICROS, SERIAL_TIMEOUT_MS,
};

use crate::error::{AvrError, AvrResult};
use std::io::{Read, Write};

pub type ComPort = String;
pub type BaudRate = u32;
/// Serial port device_interface layer
pub(crate) struct SerialPortDevice {
    pub serial_port: Box<dyn serialport::SerialPort>,
}

impl SerialPortDevice {
    pub fn new(port: ComPort, baud: BaudRate) -> AvrResult<SerialPortDevice> {
        let serial_port = serialport::new(port, baud)
            .timeout(std::time::Duration::from_millis(SERIAL_TIMEOUT_MS))
            .dtr_on_open(false)
            .open()
            .map_err(|e| AvrError::Communication(format!("{:?}", e)))?;

        Ok(SerialPortDevice { serial_port })
    }
}

impl DeviceInterface for SerialPortDevice {
    fn send(&mut self, command: Vec<u8>) -> AvrResult<()> {
        self.serial_port
            .write_all(&command)
            .map_err(|e| AvrError::Communication(format!("{:?}", e)))?;
        Ok(())
    }

    fn receive(&mut self) -> AvrResult<Vec<u8>> {
        let mut buffer: Vec<u8> = vec![0; MAX_RESPONSE_SIZE];

        let size = self
            .serial_port
            .read(&mut buffer)
            // Timeout error is fine, just continue
            .or_else(|e| {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    Ok(0)
                } else {
                    Err(e)
                }
            })
            .map_err(|e| AvrError::Communication(format!("{:?}", e)))?;

        // Return a buffer with the actual length
        buffer.truncate(size);
        Ok(buffer)
    }

    fn reset(&mut self) -> AvrResult<()> {
        // Reset logic for the serial port
        self.serial_port
            .write_data_terminal_ready(false)
            .map_err(|e| AvrError::Communication(format!("Failed to set DTR false: {:?}", e)))?;
        self.serial_port
            .write_request_to_send(false)
            .map_err(|e| AvrError::Communication(format!("Failed to set RTS false: {:?}", e)))?;

        std::thread::sleep(std::time::Duration::from_micros(RESET_DTR_RTS_LOW_MICROS));

        self.serial_port
            .write_data_terminal_ready(true)
            .map_err(|e| AvrError::Communication(format!("Failed to set DTR true: {:?}", e)))?;
        self.serial_port
            .write_request_to_send(true)
            .map_err(|e| AvrError::Communication(format!("Failed to set RTS true: {:?}", e)))?;

        std::thread::sleep(std::time::Duration::from_millis(POST_RESET_BOOTUP_DELAY_MS));
        Ok(())
    }
}
