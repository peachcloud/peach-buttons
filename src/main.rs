extern crate crossbeam_channel;
extern crate sysfs_gpio;

use std::sync::Arc;
use std::thread;

use sysfs_gpio::{Direction, Edge, Pin};

use jsonrpc_core::futures::Future;
use jsonrpc_core::*;
use jsonrpc_pubsub::{PubSubHandler, Session, Subscriber, SubscriptionId};
use jsonrpc_ws_server::{RequestContext, ServerBuilder};

use crossbeam_channel::unbounded;
use crossbeam_channel::Sender;

// initialize gpio pin and poller
// send button code to "subscribe_buttons" rpc method for sink notification
fn interrupt(pin: u64, button_code: u8, s: Sender<u8>) -> sysfs_gpio::Result<()> {
    let input = Pin::new(pin);
    input.with_exported(|| {
        input.set_direction(Direction::In)?;
        input.set_edge(Edge::FallingEdge)?;
        let mut poller = input.get_poller()?;
        loop {
            // needs debounce logic!
            match poller.poll(1000)? {
                Some(_value) => s.send(button_code).unwrap(),
                None => (), //return //println!("{:}", "".to_string())
            };
        }
    })
}

// spawn a thread to create an interrupt on a single pin
fn spawn_interrupt(pin: u64, button_code: u8, s: Sender<u8>) {
    thread::spawn(move || match interrupt(pin, button_code, s) {
        Ok(()) => println!("Interrupting Complete!"),
        Err(err) => println!("Error: {}", err),
    });
}

fn main() {
    // create channel for message passing
    let (s, r) = unbounded();
    let (s1, r1) = (s.clone(), r.clone());

    // center joystick
    spawn_interrupt(462, 0, s1);

    let s1 = s.clone();

    // left joystick
    spawn_interrupt(485, 1, s1);

    let s1 = s.clone();

    // right joystick
    spawn_interrupt(481, 2, s1);

    let s1 = s.clone();

    // up joystick
    spawn_interrupt(475, 3, s1);

    let s1 = s.clone();

    // down joystick
    spawn_interrupt(480, 4, s1);

    let s1 = s.clone();

    // A `#5`
    spawn_interrupt(463, 5, s1);

    let s1 = s.clone();

    // B `#6`
    spawn_interrupt(464, 6, s1);

    let mut io = PubSubHandler::new(MetaIoHandler::default());

    io.add_subscription(
        "button_press",
        (
            "subscribe_buttons",
            move |params: Params, _, subscriber: Subscriber| {
                if params != Params::None {
                    subscriber
                        .reject(Error {
                            code: ErrorCode::ParseError,
                            message: "Invalid parameters. Subscription rejected".into(),
                            data: None,
                        })
                        .unwrap();
                    return;
                }

                // useful for debugging: subscription request received
                println!("{:}", "Subscription event".to_string());
                let r1 = r1.clone();
                thread::spawn(move || {
                    let sink = subscriber
                        .assign_id_async(SubscriptionId::Number(1))
                        .wait()
                        .unwrap();

                    loop {
                        // listen for gpio interrupt event message
                        let button_code = r1.recv().unwrap();
                        // emit button_code to subscriber
                        match sink
                            .notify(Params::Array(vec![Value::Number(button_code.into())]))
                            .wait()
                        {
                            Ok(_) => {}
                            Err(_) => {
                                println!("Subscription has ended, finishing.");
                                break;
                            }
                        }
                    }
                });
            },
        ),
        ("remove_buttons", |_id: SubscriptionId, _| {
            println!("Closing subscription");
            futures::future::ok(Value::Bool(true))
        }),
    );

    // build the json-rpc-over-websockets server
    let server = ServerBuilder::with_meta_extractor(io, |context: &RequestContext| {
        Arc::new(Session::new(context.sender().clone()))
    })
    .start(&"127.0.0.1:3030".parse().unwrap())
    .expect("Unable to start RPC server");

    server.wait().unwrap();
}
