use crate::error::AvrResult;

pub(crate) trait TransportLayer {
    /// Send a command to the target device
    fn send(&mut self, command: Vec<u8>) -> AvrResult<()>;

    /// Receive a response from the target device
    fn receive(&mut self, expected_bytes: usize) -> AvrResult<Vec<u8>>;
}
