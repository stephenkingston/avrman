pub mod serialport;
use serialport::{BaudRate, ComPort};

use crate::error::AvrResult;

pub(crate) trait DeviceInterface {
    /// Send a command to the target device
    fn send(&mut self, command: Vec<u8>) -> AvrResult<()>;

    /// Receive a response from the target device
    fn receive(&mut self) -> AvrResult<Vec<u8>>;

    /// Reset the target device
    fn reset(&mut self) -> AvrResult<()>;
}

#[derive(Debug, Clone)]
pub struct ComPortParams {
    pub port: Option<ComPort>,

    /// Baud rate is optional, since this is usually fixed for
    /// a given microcontroller type
    pub baud: Option<BaudRate>,
}

#[derive(Debug, Clone)]
pub enum DeviceInterfaceType {
    VirtualComPort(ComPortParams),
}
