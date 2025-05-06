use std::{fs::File, io::Read};

pub use boards::Microcontroller;
use boards::protocol_for_mcu;
use error::{AvrError, AvrResult};
use ihex::Reader;
use interface::DeviceInterfaceType;
use protocols::{ProgrammerTrait, stk500v1::Stk500v1Params};

pub mod boards;
pub(crate) mod constants;
pub mod error;
pub mod interface;
pub mod protocols;
pub(crate) mod util;

pub enum ProtocolType {
    Stk500v1(Stk500v1Params),
}

pub struct Programmer {
    programmer: Box<dyn ProgrammerTrait>,
    verify: bool,
    progress_bar_enable: bool,
}

impl Programmer {
    /// Simplest method to create a new programmer. Given a supported microcontroller name,
    /// this will automatically look for the device and set all appropriate settings
    /// necessary for programming
    pub fn new(mcu: Microcontroller) -> AvrResult<Self> {
        let protocol = protocol_for_mcu(mcu, None)?;
        Self::from_protocol(protocol)
    }

    /// Create a programmer with a specific set of protocol parameters. This is can be used to program boards
    /// for which there is no official support on avrman, that use the Stk500v1 protocol
    pub fn from_protocol(protocol: ProtocolType) -> AvrResult<Self> {
        let programmer: Box<dyn ProgrammerTrait> = match protocol {
            ProtocolType::Stk500v1(params) => Box::new(protocols::stk500v1::Stk500v1::new(params)?),
        };

        Ok(Programmer {
            programmer,
            progress_bar_enable: false,
            verify: true,
        })
    }

    /// Create a programmer for a given MCU, with interface parameters (eg: for a COM port,
    /// this will be serial port and baud rate). Useful in case, Programmer::new isn't able
    /// to automatically select the serial port
    pub fn from_mcu_and_interface(
        mcu: Microcontroller,
        interface: DeviceInterfaceType,
    ) -> AvrResult<Self> {
        let protocol = protocol_for_mcu(mcu, Some(interface))?;
        Self::from_protocol(protocol)
    }

    /// Enable or disable a progress bar during programming/verify
    /// Progress bar is disabled by default
    pub fn progress_bar(&mut self, enable: bool) {
        self.progress_bar_enable = enable;
    }

    /// Enable or disable verification after programming
    /// Enabled by default
    pub fn verify_after_programming(&mut self, enable: bool) {
        self.verify = enable;
    }

    /// Parse intel hex file raw string to binary
    fn parse_intel_hex(&self, hex_content: &str) -> AvrResult<Vec<u8>> {
        let mut bin = Vec::new();
        let parser = Reader::new(hex_content);
        for record in parser {
            match record {
                Ok(rec) => {
                    if let ihex::Record::Data { value, .. } = rec {
                        bin.extend_from_slice(&value);
                    }
                }
                Err(e) => {
                    return Err(AvrError::ProgrammerError(format!(
                        "Failed parsing record in hex file {:?}",
                        e
                    )));
                }
            }
        }

        Ok(bin)
    }

    /// Program board with provided intelhex file from file path
    pub fn program_hex_file(&self, file_path: &str) -> AvrResult<()> {
        let mut file = File::open(file_path)
            .map_err(|e| AvrError::FirmwareError(format!("Failed to read file: {}", e)))?;
        let mut hex_content = String::new();
        file.read_to_string(&mut hex_content).map_err(|e| {
            AvrError::FirmwareError(format!("Could not read given hex file to string {:?}", e))
        })?;

        let bin = self.parse_intel_hex(&hex_content)?;
        self.programmer
            .program_firmware(bin, self.verify, self.progress_bar_enable)?;

        Ok(())
    }

    /// Program provided intelhex, input as string read from a .hex file
    pub fn program_hex_buffer(&self, hex_content: &str) -> AvrResult<()> {
        let bin = self.parse_intel_hex(hex_content)?;
        self.programmer
            .program_firmware(bin, self.verify, self.progress_bar_enable)?;
        Ok(())
    }

    /// Program binary data
    pub fn program_binary(&self, bin: Vec<u8>) -> AvrResult<()> {
        self.programmer
            .program_firmware(bin, self.verify, self.progress_bar_enable)?;
        Ok(())
    }
}
