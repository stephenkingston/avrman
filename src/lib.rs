use error::{AvrError, AvrResult};

pub(crate) mod constants;
pub mod error;
pub mod protocols;
pub(crate) mod transport;
pub(crate) mod util;

pub enum ProtocolType {
    Stk500 { serial_port: String, baud_rate: u32 },
}

pub struct Programmer {
    programmer: Box<dyn ProgrammerTrait>,
}

pub(crate) trait ProgrammerTrait {
    fn program_firmware(&self, firmware: Vec<u8>) -> AvrResult<()>;
    fn reset(&self) -> AvrResult<()>;
}

impl Programmer {
    pub fn new(protocol: ProtocolType) -> AvrResult<Self> {
        let programmer: Box<dyn ProgrammerTrait> = match &protocol {
            ProtocolType::Stk500 {
                serial_port,
                baud_rate,
            } => Box::new(protocols::stk500::Stk500::new(
                serial_port.clone(),
                *baud_rate,
            )?),
        };

        Ok(Programmer { programmer })
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
