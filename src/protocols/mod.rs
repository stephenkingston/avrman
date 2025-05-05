use crate::error::AvrResult;
pub mod stk500v1;

/// Currently only implements program_firmware/reset. Can be extended in
/// future to do other operations like dump flash, erase chip, etc.,
pub(crate) trait ProgrammerTrait {
    fn program_firmware(&self, firmware: Vec<u8>) -> AvrResult<()>;
    fn reset(&self) -> AvrResult<()>;
}
