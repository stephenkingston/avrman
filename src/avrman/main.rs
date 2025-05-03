use avrman::error::AvrResult;

fn main() -> AvrResult<()> {
    let protocol = avrman::ProtocolType::Stk500 {
        serial_port: String::from("/dev/ttyUSB0"),
        baud_rate: 115200,
    };

    let programmer = avrman::Programmer::new(protocol)?;
    programmer.program_file("./tests/blink.uno.hex")?;

    Ok(())
}
