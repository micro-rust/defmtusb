//! Main task that runs the USB transport layer.



#![allow(unused_labels)]



use embassy_usb::{
    Config,

    class::cdc_acm::{
        Sender, State,
    },

    driver::Driver,
};

use static_cell::StaticCell;



/// Statically allocated device descriptor buffer.
#[link_section = ".bss.defmtusb.DESCRIPTORS"]
static mut DEVBUF: [u8; 256] = [0u8; 256];

/// Statically allocated configuration descriptor buffer.
#[link_section = ".bss.defmtusb.DESCRIPTORS"]
static mut CFGBUF: [u8; 256] = [0u8; 256];

/// Statically allocated BOS descriptor buffer.
#[link_section = ".bss.defmtusb.DESCRIPTORS"]
static mut BOSBUF: [u8; 256] = [0u8; 256];

/// Statically allocated control buffer.
#[link_section = ".bss.defmtusb.DESCRIPTORS"]
static mut CONTROL: [u8; 256] = [0u8; 256];

/// Statically allocated CDC ACM state.
static STATE: StaticCell<State> = StaticCell::new();



/// Builds the USB class and runs both the logger and USB.
/// Requires the USB driver provided by the HAL and the maximum packet size
/// allowed in the device.
/// The user may provide an optional USB configuration to set the VID, PID and
/// other information of the USB device. If none is provided a default
/// configuration will be set.
pub async fn run<D: Driver<'static>>(driver: D, size: usize, config: Option<Config<'static>>) {
    use embassy_usb::{
        Builder,
    
        class::cdc_acm::CdcAcmClass,
    };

    // Create the configuration.
    let config = match config {
        // Set default configuration.
        None => {
            // Create the configuration.
            let mut cfg = Config::new(0xDEF7, 0xDA7A);

            // Set information strings.
            cfg.manufacturer = Some("micro-rust organization");
            cfg.product = Some("USB defmt logger");
            cfg.serial_number = Some("314159");

            // Configure the default max power.
            cfg.max_power = 100;

            // Configure the max packet size.
            cfg.max_packet_size_0 = size as u8;

            cfg
        },

        // User provided configuration.
        Some(c) => c,
    };

    // Configure the device class.
    config.device_class = 0x02;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;

    // Get the static buffers.
    let (devdesc, cfgdesc, bosdesc, ctrlbuf) = unsafe {(
        &mut DEVBUF,
        &mut CFGBUF,
        &mut BOSBUF,
        &mut CONTROL,
    )};

    // Create the state of the CDC ACM device.
    let state: &'static mut State<'static> = STATE.init( State::new() );

    // Create the USB builder.
    let mut builder = Builder::new(
        driver,
        config,
        devdesc,
        cfgdesc,
        bosdesc,
        ctrlbuf,
    );

    // Create the class on top of the builder.
    let class = CdcAcmClass::new(&mut builder, state, size as u16);

    // Build the USB.
    let mut usb = builder.build();

    // Get the sender.
    let (sender, _) = class.split();

    // Run both futures concurrently.
    embassy_futures::join::join(usb.run(), logger(sender, size as usize)).await;
}



/// Runs the logger task.
pub async fn logger<'d, D: Driver<'d>>(mut sender: Sender<'d, D>, size: usize) {
    use embassy_time::{
        Timer, Duration,
    };

    use embassy_usb::driver::EndpointError;
    
    // Get a reference to the controller.
    let controller = unsafe { &mut super::controller::CONTROLLER };

    // Get a reference to the buffers.
    let buffers = unsafe { &mut super::controller::BUFFERS };

    'main : loop {
        // Wait for the device to be connected.
        sender.wait_connection().await;

        // Set the controller as enabled.
        controller.enable();

        // Begin sending the data.
        'data : loop {

            // Wait for new data.
            let buffer = 'select : loop {
                // Check which buffer is flushing.
                if buffers[0].flushing() { break 'select &mut buffers[0]; }
                if buffers[1].flushing() { break 'select &mut buffers[1]; }

                // Wait the timeout.
                // TODO : Make this configurable.
                Timer::after( Duration::from_millis(100) ).await;
            };

            // Get an iterator over the data of the buffer.
            let chunks = buffer.data[..buffer.cursor].chunks(size);

            for chunk in chunks {
                // Send the data.
                match sender.write_packet(chunk).await {
                    Err(e) => match e {
                        // The endpoint was disconnected.
                        EndpointError::Disabled => {
                            // Reset the buffer as its contents' integrity is gone.

                            // Disable the controller.
                            controller.disable();

                            continue 'main;
                        },

                        _ => (),
                    },

                    _ => (),
                }
            }

            // Reset the buffer as it has been transmitted.
            buffer.reset();
        }
    }
}
