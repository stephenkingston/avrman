use std::path::PathBuf;

use avrman::{
    Microcontroller,
    error::AvrResult,
    interface::{ComPortParams, DeviceInterfaceType},
};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub(crate) struct ProgramOptions {
    /// Board type
    #[clap(short, long)]
    board: Microcontroller,

    /// Firmware
    #[clap(short, long)]
    firmware: PathBuf,

    /// Serial port
    #[clap(short, long)]
    serial: Option<String>,

    /// Baud rate
    #[clap(short, long)]
    baudrate: Option<u32>,

    #[clap(short, long, default_value_t = false)]
    no_verify: bool,
}

pub(crate) fn handle_programming(opts: ProgramOptions) -> AvrResult<()> {
    let mcu = opts.board;
    let file = opts.firmware;

    let mut programmer = if opts.serial.is_some() || opts.baudrate.is_some() {
        let interface = DeviceInterfaceType::VirtualComPort(ComPortParams {
            port: opts.serial,
            baud: opts.baudrate,
        });
        avrman::Programmer::from_mcu_and_interface(mcu, interface)?
    } else {
        avrman::Programmer::new(mcu)?
    };

    programmer.progress_bar(true);
    programmer.verify_after_programming(!opts.no_verify);

    programmer.program_hex_file(
        file.to_str()
            .expect("Could not convert firmware PathBuf to string"),
    )?;

    Ok(())
}
