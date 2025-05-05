use crate::{ProtocolType, Stk500v1Params, interface::serialport::ComPort};

/// Microcontroller enum includes all boards/microcontrollers
/// that have been tested to work with avrman
pub enum Microcontroller {
    /// ATMega328p/Arduino Uno, initialized with COM/serial port
    ArduinoUno(ComPort),
}

pub(crate) fn protocol_for_mcu(board: Microcontroller) -> ProtocolType {
    match board {
        Microcontroller::ArduinoUno(port) => ProtocolType::Stk500v1(Stk500v1Params {
            port,
            baud: 115200,
            signature: vec![0x1e, 0x95, 0x0f],
            page_size: 128,
            num_pages: 256,
            product_id: vec![0x0043, 0x7523, 0x0001, 0xea60, 0x6015],
            verify: true,
        }),
    }
}
