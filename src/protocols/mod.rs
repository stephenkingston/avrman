use crate::error::AvrResult;
pub mod stk500v1;
pub mod stk500v2;

/// Currently only implements program/reset. Can be extended in
/// future to do other operations like dump flash, erase chip, etc.,
pub(crate) trait ProgrammerTrait {
    fn program_firmware(
        &self,
        firmware: Vec<u8>,
        verify: bool,
        enable_progress_bar: bool,
    ) -> AvrResult<()>;
    fn reset(&self) -> AvrResult<()>;
}
