use avrman::{Microcontroller, error::AvrResult};

fn main() -> AvrResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mcu = Microcontroller::ArduinoUno(String::from("/dev/ttyUSB0"));
    let mut programmer = avrman::Programmer::new(mcu)?;
    programmer.progress_bar(true);

    programmer.program_hex_file("./tests/etp.hex")?;

    Ok(())
}
