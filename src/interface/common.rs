use crate::error::AvrResult;

pub(crate) trait ProgrammerInterface {
    /// Send a command to the target device
    fn send(&mut self, command: Vec<u8>) -> AvrResult<()>;

    /// Receive a response from the target device
    fn receive(&mut self) -> AvrResult<Vec<u8>>;

    /// Reset the target device
    fn reset(&mut self) -> AvrResult<()>;
}
