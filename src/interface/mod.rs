pub mod serialport;

use crate::error::AvrResult;

pub(crate) trait DeviceInterface {
    /// Send a command to the target device
    fn send(&mut self, command: Vec<u8>) -> AvrResult<()>;

    /// Receive a response from the target device
    fn receive(&mut self) -> AvrResult<Vec<u8>>;

    /// Flush send/receive buffers
    fn flush_buffers(&mut self) -> AvrResult<()>;

    /// Reset the target device
    fn reset(&mut self) -> AvrResult<()>;
}
