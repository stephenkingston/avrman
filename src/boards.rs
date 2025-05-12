use clap::ValueEnum;

use crate::{
    ProtocolType, Stk500v1Params,
    error::{AvrError, AvrResult},
    interface::DeviceInterfaceType,
    protocols::stk500v2::Stk500v2Params,
};

/// Microcontroller enum includes all boards/microcontrollers
/// that have been tested to work with avrman
#[derive(Debug, Clone, ValueEnum)]
pub enum Microcontroller {
    /// Atmega328p initialized with COM/serial port
    ArduinoUno,

    /// Same as Arduino Uno
    Atmega328p,

    /// Arduino Nano
    ArduinoNano,

    /// Arduino Mega
    ArduinoMega,
}

/// Figure out the protocol and all associated parameters for a given MCU
/// interface_type can be provided to override default parameters/make it
/// easier to complete the DeviceInterfaceType enum
/// For instance if the serial port is not provided, this function will
/// attempt to find the serial port where the given MCU is connected
pub fn protocol_for_mcu(
    mcu: Microcontroller,
    interface_type: Option<DeviceInterfaceType>,
) -> AvrResult<ProtocolType> {
    match mcu {
        Microcontroller::ArduinoUno | Microcontroller::Atmega328p => {
            let default_baud_rate = 115200;
            let signature = vec![0x1e, 0x95, 0x0f];
            let page_size = 128;
            let num_pages = 256;
            let product_id = vec![0x0043, 0x7523, 0x0001, 0xea60, 0x6015];

            let (port, baud) = match interface_type {
                Some(interface) => {
                    let DeviceInterfaceType::Serial(params) = interface;
                    let port = params
                        .port
                        .unwrap_or(serial_port_from_product_id(&product_id)?);
                    (port, params.baud.unwrap_or(default_baud_rate))
                }
                None => {
                    // Default baud rate when none is provided
                    let baud = default_baud_rate;

                    // Try to find the serial port using product_id
                    let port = serial_port_from_product_id(&product_id)?;

                    (port, baud)
                }
            };

            Ok(ProtocolType::Stk500v1(Stk500v1Params {
                port,
                baud,
                device_signature: signature,
                page_size,
                num_pages,
                product_id,
            }))
        }
        Microcontroller::ArduinoNano => {
            let default_baud_rate = 57600;
            let signature = vec![0x1e, 0x95, 0x0f];
            let page_size = 128;
            let num_pages = 256;
            let product_id = vec![0x6001, 0x7523];

            let (port, baud) = match interface_type {
                Some(interface) => {
                    let DeviceInterfaceType::Serial(params) = interface;
                    let port = params
                        .port
                        .unwrap_or(serial_port_from_product_id(&product_id)?);
                    (port, params.baud.unwrap_or(default_baud_rate))
                }
                None => {
                    // Default baud rate when none is provided
                    let baud = default_baud_rate;

                    // Try to find the serial port using product_id
                    let port = serial_port_from_product_id(&product_id)?;

                    (port, baud)
                }
            };

            Ok(ProtocolType::Stk500v1(Stk500v1Params {
                port,
                baud,
                device_signature: signature,
                page_size,
                num_pages,
                product_id,
            }))
        }
        Microcontroller::ArduinoMega => {
            let default_baud_rate = 115200;
            let signature = vec![0x1e, 0x98, 0x01];
            let page_size = 256;
            let product_id = vec![0x6001, 0x7523];

            let (port, baud) = match interface_type {
                Some(interface) => {
                    let DeviceInterfaceType::Serial(params) = interface;
                    let port = params
                        .port
                        .unwrap_or(serial_port_from_product_id(&product_id)?);
                    (port, params.baud.unwrap_or(default_baud_rate))
                }
                None => {
                    // Default baud rate when none is provided
                    let baud = default_baud_rate;

                    // Try to find the serial port using product_id
                    let port = serial_port_from_product_id(&product_id)?;

                    (port, baud)
                }
            };

            Ok(ProtocolType::Stk500v2(Stk500v2Params {
                port,
                baud,
                device_signature: signature,
                page_size,
                product_id,
            }))
        }
    }
}

pub(crate) fn serial_port_from_product_id(product_ids: &Vec<u16>) -> AvrResult<String> {
    match serialport::available_ports() {
        Ok(ports) => {
            for port in ports {
                if let serialport::SerialPortType::UsbPort(info) = port.port_type {
                    for pid in product_ids {
                        if *pid == info.pid {
                            return Ok(port.port_name);
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(AvrError::ConfigurationError(format!(
                "Could not get available ports. Err {:?}",
                e
            )));
        }
    };

    Err(AvrError::ConfigurationError(format!(
        "Looked at all available serial ports; could not find one that matches one of 
        product IDs {:?}. Try specifying a serial port for the given MCU?",
        product_ids
    )))
}
