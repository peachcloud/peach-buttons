extern crate crossbeam_channel;
extern crate gpio_cdev;

use std::{cell::Cell, process, thread, time::Duration};

use crossbeam_channel::{tick, Sender};
use gpio_cdev::{Chip, LineRequestFlags};

// initialize gpio pin and poll for state (debounced 1ms)
// send button code to "subscribe_buttons" rpc method for sink notification
pub fn interrupt_handler(pin: u32, button_code: u8, button_name: String, s: Sender<u8>) {
    thread::spawn(move || {
        debug!("Creating handle for GPIO chip.");
        let mut chip = Chip::new("/dev/gpiochip0").unwrap_or_else(|err| {
            error!("Failed to create handle for GPIO chip: {}", err);
            process::exit(1);
        });

        debug!("Creating handle for GPIO line at given pin.");
        let input = chip.get_line(pin).unwrap_or_else(|err| {
            error!(
                "Failed to create handle for GPIO line at pin {}: {}",
                pin, err
            );
            process::exit(1);
        });

        let line_handle = input
            .request(LineRequestFlags::INPUT, 0, &button_name)
            .unwrap_or_else(|err| {
                error!("Failed to gain kernel access for pin {}: {}", pin, err);
                process::exit(1);
            });

        let ticker = tick(Duration::from_millis(2));
        let mut counter = Cell::new(0);
        let mut switch = Cell::new(0);

        info!(
            "Initating polling loop for {} button on pin {}",
            button_name, pin
        );
        loop {
            ticker.recv().unwrap();
            let value = line_handle.get_value().unwrap();
            match value {
                0 => counter.set(0),
                1 => *counter.get_mut() += 1,
                _ => (),
            }
            if counter.get() == 10 {
                if switch.get() == 0 {
                    *switch.get_mut() += 1
                } else {
                    debug!("Sending button code: {}", button_code);
                    s.send(button_code).unwrap_or_else(|err| {
                        error!("Failed to send button_code to publisher: {}", err);
                    });
                }
            }
        }
    });
}
