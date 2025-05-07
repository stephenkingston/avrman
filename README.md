# avrman

avrman is a programmer for AVR microcontrollers written natively in Rust. Right
now, this has only been tested to support Arduino Uno (ATMega328p), but it could
also work with other AVR microcontrollers that use the STK500v1 protocol.

avrman can be used as both a library or with it's standalone `avrman`
executable.

## Usage as an executable

To install avrman globally as an executable, run the following cargo command:

```
cargo install avrman
```

Now, you can execute `avrman` from any terminal

```
> avrman
Usage: avrman <COMMAND>

Commands:
  program  Program target device with options
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

To program an Arduino Uno, you can now run

```
> avrman program -b arduino-uno -f ~/repos/avrman/tests/blink.hex
◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼ 8/8 (100%) Programmed.
◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼◼ 8/8 (100%) Verified.
Done! ✨ 🍰 ✨

```

> **Note**
>
> This command includes optional `--serial` and `--baudrate` arguments which are
> picked automatically based on the provided microcontroller/board name.

## Usage as a library

You can use avrman in your own Rust code as a library.

```
cargo add avrman
```

To use a tested and supported board:

```rust

fn main() -> AvrResult<()> {
    let mut programmer = Programmer::new(avrman::Microcontroller::ArduinoUno)?;
    programmer.progress_bar(true); // Optional, shows a progress bar during programming
    programmer.verify_after_programming(false); // Optional, disable verify

    programmer.program_hex_file("./blink.hex")?;
}

```

### Advanced

To use a board that uses Stk500v1 protocol and if you are aware of all the
parameters necessary to make it work, use this.

```rust
fn main() -> AvrResult<()> {
    let mut programmer =
        Programmer::from_protocol(ProtocolType::Stk500v1(Stk500v1Params {
            port,
            baud: 115200,
            device_signature: vec![0x1e, 0x95, 0x0f],
            page_size: 128,
            num_pages: 256,
            product_id: vec![0x0043, 0x7523, 0x0001, 0xea60, 0x6015],
        }))?;

    programmer.progress_bar(true);
    programmer.verify_after_programming(false);
    programmer.program_hex_file("./hello_world.hex")?;
}

```
