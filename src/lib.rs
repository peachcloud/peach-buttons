#[macro_use]
pub extern crate log;
extern crate env_logger;
extern crate crossbeam_channel;
extern crate gpio_cdev;

use std::thread;
use std::process;
use std::sync::Arc;
use std::error::Error;
use std::result::Result;

use gpio_cdev::*;

use jsonrpc_core::futures::Future;
use jsonrpc_core::*;
use jsonrpc_pubsub::{PubSubHandler, Session, Subscriber, SubscriptionId};
use jsonrpc_ws_server::{RequestContext, ServerBuilder};

use crossbeam_channel::unbounded;
use crossbeam_channel::Sender;

// initialize gpio pin and listen for line events
// send button code to "subscribe_buttons" rpc method for sink notification
pub fn interrupt_handler(pin: u32, button_code: u8, button_name: String, s: Sender<u8>) {
    thread::spawn(move || {
        debug!("Creating handle for GPIO chip.");
        let mut chip = Chip::new("/dev/gpiochip0").unwrap_or_else(|err| {
            error!("Problem creating handle for GPIO chip: {}", err);
            process::exit(1);
        });

        debug!("Creating handle for GPIO line at given pin.");
        let input = chip.get_line(pin).unwrap_or_else(|err| {
            error!("Problem creating handle for GPIO line: {}", err);
            process::exit(1);
        });
    
        info!("Listening for line events on pin: {}", pin);
        for _event in input.events(
            LineRequestFlags::INPUT,
            EventRequestFlags::FALLING_EDGE,
            &button_name
        ).unwrap() {
            debug!("Sending button code to publisher: {}", &button_code);
            s.send(button_code).unwrap_or_else(|err| {
                error!("Problem sending button code to publisher: {}", err);
                process::exit(1);
            });
        }
    });
}        

pub fn run() -> Result<(), Box<dyn Error>> {
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
                        .unwrap();
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