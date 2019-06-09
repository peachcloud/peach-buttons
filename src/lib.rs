#[macro_use]
pub extern crate log;
extern crate crossbeam_channel;
extern crate gpio_cdev;

use std::error;
use std::process;
use std::result::Result;
use std::sync::Arc;
use std::thread;

use gpio_cdev::{Chip, EventRequestFlags, LineRequestFlags};

use jsonrpc_core::futures::Future;
use jsonrpc_core::{ErrorCode, *};
use jsonrpc_pubsub::{PubSubHandler, Session, Subscriber, SubscriptionId};
#[allow(unused_imports)]
use jsonrpc_test as test;
use jsonrpc_ws_server::{RequestContext, ServerBuilder};

use crossbeam_channel::{unbounded, Sender};

pub type BoxError = Box<dyn error::Error>;

// initialize gpio pin and listen for line events
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

        info!("Listening for line events on pin: {}", pin);
        for _event in input
            .events(
                LineRequestFlags::INPUT,
                EventRequestFlags::FALLING_EDGE,
                &button_name,
            )
            .unwrap_or_else(|err| {
                error!("Failed to create handle for line events: {}", err);
                process::exit(1);
            })
        {
            debug!("Sending button code to publisher: {}", &button_code);
            s.send(button_code).unwrap_or_else(|err| {
                error!(
                    "Failed to send message because the channel is disconnected: {}",
                    err
                );
                process::exit(1);
            });
        }
    });
}

pub fn run() -> Result<(), BoxError> {
    info!("Starting up.");

    debug!("Creating channel for message passing.");
    // create channel for message passing
    let (s, r) = unbounded();
    let (s1, r1) = (s.clone(), r.clone());

    debug!("Setting up interrupt handlers.");
    // center joystick
    interrupt_handler(4, 0, "center".to_string(), s1);

    let s1 = s.clone();

    // left joystick
    interrupt_handler(27, 1, "left".to_string(), s1);

    let s1 = s.clone();

    // right joystick
    interrupt_handler(23, 2, "right".to_string(), s1);

    let s1 = s.clone();

    // up joystick
    interrupt_handler(17, 3, "up".to_string(), s1);

    let s1 = s.clone();

    // down joystick
    interrupt_handler(22, 4, "down".to_string(), s1);

    let s1 = s.clone();

    // A `#5`
    interrupt_handler(5, 5, "#5".to_string(), s1);

    let s1 = s.clone();

    // B `#6`
    interrupt_handler(6, 6, "#6".to_string(), s1);

    debug!("Creating pub-sub handler.");
    let mut io = PubSubHandler::new(MetaIoHandler::default());

    io.add_subscription(
        "button_press",
        (
            "subscribe_buttons",
            move |params: Params, _, subscriber: Subscriber| {
                if params != Params::None {
                    debug!("Received subscription request.");
                    subscriber
                        .reject(jsonrpc_core::Error {
                            code: ErrorCode::ParseError,
                            message: "Invalid parameters. Subscription rejected".into(),
                            data: None,
                        })
                        .unwrap_or_else(|_| {
                            error!("Failed to send rejection error for subscription request");
                        });
                    return;
                }

                let r1 = r1.clone();
                thread::spawn(move || {
                    let sink = subscriber
                        .assign_id_async(SubscriptionId::Number(1))
                        .wait()
                        .unwrap();

                    info!("Listening for button code from gpio events.");
                    loop {
                        // listen for gpio interrupt event message
                        let button_code: u8 = r1.recv().unwrap();
                        info!("Received button code: {}.", button_code);
                        // emit button_code to subscriber
                        match sink
                            .notify(Params::Array(vec![Value::Number(button_code.into())]))
                            .wait()
                        {
                            Ok(_) => {
                                info!("Publishing button code to subscriber over ws.");
                            }
                            Err(_) => {
                                // subscription terminated due to error
                                warn!("Failed to publish button code.");
                                break;
                            }
                        }
                    }
                });
            },
        ),
        ("remove_buttons", |_id: SubscriptionId, _| {
            // unsubscribe
            futures::future::ok(Value::Bool(true))
        }),
    );

    info!("Creating JSON-RPC server.");
    // build the json-rpc-over-websockets server
    let server = ServerBuilder::with_meta_extractor(io, |context: &RequestContext| {
        Arc::new(Session::new(context.sender().clone()))
    })
    .start(&"127.0.0.1:3030".parse().unwrap())
    .expect("Unable to start RPC server");

    info!("Listening for requests.");
    server.wait().unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_success() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_success_response", |_| {
                Ok(Value::String("success".into()))
            });
            test::Rpc::from(io)
        };

        assert_eq!(rpc.request("rpc_success_response", &()), r#""success""#);
    }
}
