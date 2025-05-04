use crate::{ProtocolType, Stk500Params, interface::serialport::ComPort};

/// Includes all boards/microcontrollers that have been tested to work
pub enum Microcontroller {
    /// ATMega328p (Arduino Uno)
    ArduinoUno(ComPort),
}

pub fn protocol_for_mcu(board: Microcontroller) -> ProtocolType {
    match board {
        Microcontroller::ArduinoUno(port) => ProtocolType::Stk500(Stk500Params {
            port,
            baud: 115200,
            signature: vec![0x1e, 0x95, 0x0f],
        }),
    }
}
