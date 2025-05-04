use boards::protocol_for_mcu;
use error::{AvrError, AvrResult};
use protocols::stk500::Stk500Params;

pub mod boards;
pub(crate) mod constants;
pub mod error;
pub(crate) mod interface;
pub mod protocols;
pub(crate) mod util;

pub use boards::Microcontroller;

pub enum ProtocolType {
    Stk500(Stk500Params),
}

pub struct Programmer {
    programmer: Box<dyn ProgrammerTrait>,
}

pub(crate) trait ProgrammerTrait {
    fn program_firmware(&self, firmware: Vec<u8>) -> AvrResult<()>;
    fn reset(&self) -> AvrResult<()>;
}

impl Programmer {
    pub fn from_protocol(protocol: ProtocolType) -> AvrResult<Self> {
        let programmer: Box<dyn ProgrammerTrait> = match protocol {
            ProtocolType::Stk500(params) => Box::new(protocols::stk500::Stk500::new(params)?),
        };

        Ok(Programmer { programmer })
    }

    pub fn new(mcu: Microcontroller) -> AvrResult<Self> {
        let protocol = protocol_for_mcu(mcu);
        Self::from_protocol(protocol)
    }

    pub fn program_file(&self, file_path: &str) -> AvrResult<()> {
        let buffer = std::fs::read(file_path)
            .map_err(|e| AvrError::FirmwareError(format!("Failed to read file: {}", e)))?;

        self.programmer.program_firmware(buffer)?;

        Ok(())
    }

    pub fn program_buffer(&self, buffer: &[u8]) -> AvrResult<()> {
        self.programmer.program_firmware(buffer.to_vec())?;
        Ok(())
    }
}
