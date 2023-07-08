# defmtusb

`defmtusb` is a logger implementation for the `defmt` protocol that transports the log frames over USB. It is currently implemented to run within the `embassy` framework, with plans to allow for bare metal operation in the future.

[`defmt`](https://github.com/knurling-rs/defmt) is a highly efficient logging framework targetting resource constrained devices like microcontrollers.

This API is formed by two parts, microncontroller side tooling (this library) and host side tooling ([`defmthost`](https://github.com/micro-rust/defmthost)).

## Usage

To use `defmtusb` in your application simply add to your `Cargo.toml` file

```toml
defmtusb = "*"
```

and, at the root of your file structure (`main.rs`) add the next line to include the global logger
```rust
use defmtusb as _;
```

Remember that only one `defmt` logger must be imported at any time, otherwise the compilation may fail.

## `embassy`

To run the logger within the embassy framework, there are two methods that give a different level of granularity to the user.

### Simple method

If you don't need to use the USB driver in your application, you can simply pass ownership of this driver to the `run` task in `defmtusb` as follows

```rust
#[task]
pub(super) async fn logger(usb: USB) {
    // Create the USB driver.
    let driver = Driver::new(usb, Irqs);

    defmtusb::run(driver, <max_packet_size>, None).await;
}
```

The user must insert the maximum packet size of the USB hardware, as there is no way to know this without hardware specific knowledge.

Additionally the user may provide a configuration to the `run` function in order to customize the USB configuration, although the class of the device will be hard set to CDC ACM in order to maintain compatibility with UART to USB bridges (FT232, CP2120, etc...).

```rust
#[task]
pub(super) async fn logger(usb: USB) {
    // Create the USB driver.
    let driver = Driver::new(usb, Irqs);

    // Create the configuration.
    let mut cfg = Config::new(0xCAFE, 0xBEEF);

    // Set information strings.
    cfg.manufacturer = Some("manufacturer");
    cfg.product = Some("product");
    cfg.serial_number = Some("serial");

    // Configure the default max power.
    cfg.max_power = 100;

    // Configure the max control packet size.
    cfg.max_packet_size_0 = size as u8;

    defmtusb::run(driver, <max_packet_size>, Some(cfg)).await;
}
```


### Granular method

If you intend to create a variety of endpoints in the USB and use them, you can create them and then simply pass a CDC ACM `Sender` to the `logger` task in `defmtusb`. This method also requires the maximum packet size of the hardware USB implementation.


```rust
#[task]
pub(super) async fn logger(usb: USB) {
    // Create the USB driver.
    let driver = Driver::new(usb, Irqs);

    // Create the different interfaces and endpoints.
    ...

    // Create the CDC ACM class.
    let cdc = CdcAcmClass::new(&mut builder, &mut state, <max_packet_size>);

    // Split to get the sender only.
    let sender = class.split();

    // Run only the logging function.
    defmtusb::logger(sender, <max_packet_size>).await;
}
```

## Contributing

Any contribution intentionally submitted for inclusion in the work by you shall be licensed under either the MIT License or the Mozilla Public License Version 2.0, without any additional terms and conditions.


## License
This work is licensed under

 - MIT License
 - Mozilla Public License Version 2.0 [LICENSE-MPL](/LICENSE-MPL)
 