#[cfg(test)]
mod tests {
    use avrman::{ProtocolType, interface::SerialportParams, protocols::stk500v1::Stk500v1Params};

    #[test]
    fn test_arduino_programming() {
        use avrman::Programmer;

        let blink_hex = String::from("./tests/blink.hex");
        let etp_hex = String::from("./tests/etp.hex");
        let port = "/dev/ttyUSB0".to_string();

        // Simple
        {
            let mut programmer = Programmer::new(avrman::Microcontroller::ArduinoUno).unwrap();
            programmer.verify_after_programming(false);
            programmer.program_hex_file(&blink_hex).unwrap();
        }

        // From MCU and interface
        {
            let mut programmer = Programmer::from_mcu_and_interface(
                avrman::Microcontroller::ArduinoUno,
                avrman::interface::DeviceInterfaceType::Serial(SerialportParams {
                    port: Some(port.clone()),
                    baud: Some(115200),
                }),
            )
            .unwrap();

            programmer.progress_bar(true);
            programmer.program_hex_file(&blink_hex).unwrap();
        }

        {
            // From MCU and custom protocol
            let mut programmer =
                Programmer::from_protocol(ProtocolType::Stk500v1(Stk500v1Params {
                    port,
                    baud: 115200,
                    device_signature: vec![0x1e, 0x95, 0x0f],
                    page_size: 128,
                    num_pages: 256,
                    product_id: vec![0x0043, 0x7523, 0x0001, 0xea60, 0x6015],
                }))
                .unwrap();

            programmer.progress_bar(true);
            programmer.verify_after_programming(false);
            programmer.program_hex_file(&etp_hex).unwrap();
        }
    }
}
