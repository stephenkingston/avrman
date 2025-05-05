use avrman::{Microcontroller, error::AvrResult};

fn main() -> AvrResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mcu = Microcontroller::ArduinoUno(String::from("/dev/ttyUSB0"));
    let programmer = avrman::Programmer::new(mcu)?;

    programmer.program_hex_file("./tests/blink.uno.hex")?;

    Ok(())
}
