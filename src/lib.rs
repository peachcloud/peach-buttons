#[macro_use]
pub extern crate log;
extern crate crossbeam_channel;
extern crate gpio_cdev;

mod error;
mod interrupt;

use std::{env, result::Result, sync::Arc, thread};

use crossbeam_channel::bounded;
use jsonrpc_core::futures::Future;
use jsonrpc_core::*;
use jsonrpc_pubsub::{PubSubHandler, Session, Subscriber, SubscriptionId};
#[allow(unused_imports)]
use jsonrpc_test as test;
use jsonrpc_ws_server::{RequestContext, ServerBuilder};

use crate::error::{BoxError, ButtonError::RejectSubscription};
use crate::interrupt::*;

pub fn run() -> Result<(), BoxError> {
    info!("Starting up.");

    debug!("Creating channel for message passing.");
    // create channel for message passing
    let (s, r) = bounded(0);

    debug!("Setting up interrupt handlers.");
    // center joystick
    interrupt_handler(4, 0, "center".to_string(), s.clone());

    // left joystick
    interrupt_handler(27, 1, "left".to_string(), s.clone());

    // right joystick
    interrupt_handler(23, 2, "right".to_string(), s.clone());

    // up joystick
    interrupt_handler(17, 3, "up".to_string(), s.clone());

    // down joystick
    interrupt_handler(22, 4, "down".to_string(), s.clone());

    // A `#5`
    interrupt_handler(5, 5, "#5".to_string(), s.clone());

    // B `#6`
    interrupt_handler(6, 6, "#6".to_string(), s.clone());

    debug!("Creating pub-sub handler.");
    let mut io = PubSubHandler::new(MetaIoHandler::default());

    io.add_subscription(
        "button_press",
        (
            "subscribe_buttons",
            move |params: Params, _, subscriber: Subscriber| {
                debug!("Received subscription request.");
                if params != Params::None {
                    subscriber
                        .reject(Error::from(RejectSubscription))
                        .unwrap_or_else(|_| {
                            error!("Failed to send rejection error for subscription request.");
                        });
                    return;
                }

                let r1 = r.clone();

                thread::spawn(move || {
                    let sink = subscriber
                        .assign_id_async(SubscriptionId::Number(1))
                        .wait()
                        .unwrap();

                    info!("Listening for button code from gpio events.");
                    loop {
                        let button_code: u8 = r1.recv().unwrap();
                        info!("Received button code: {}.", button_code);
                        match sink
                            .notify(Params::Array(vec![Value::Number(button_code.into())]))
                            .wait()
                        {
                            Ok(_) => info!("Publishing button code to subscriber over ws."),
                            Err(_) => {
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

    let ws_server =
        env::var("PEACH_BUTTONS_SERVER").unwrap_or_else(|_| "127.0.0.1:5111".to_string());

    info!("Starting JSON-RPC server on {}.", ws_server);
    let server = ServerBuilder::with_meta_extractor(io, |context: &RequestContext| {
        Arc::new(Session::new(context.sender().clone()))
    })
    .start(
        &ws_server
            .parse()
            .expect("Invalid WS address and port combination"),
    )
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
