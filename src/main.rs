extern crate sysfs_gpio;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;

use jsonrpc_client_http::HttpTransport;
use std::thread;
use sysfs_gpio::{Direction, Edge, Pin};

// define json-rpc server methods (to be called by this client)
jsonrpc_client!(pub struct PeachMenuClient {
    // send button code to peach-menu
    pub fn press(&mut self, button_code: u8) -> RpcRequest<String>;
});

// initialize json-rpc client with http transport
// send button code to peach_menu using json-rpc client
fn interrupt(pin: u64, button_code: u8) -> sysfs_gpio::Result<()> {
    let input = Pin::new(pin);
    let transport = HttpTransport::new().standalone().unwrap();
    let transport_handle = transport.handle("http://127.0.0.1:3031/").unwrap();
    let mut client = PeachMenuClient::new(transport_handle);
    input.with_exported(|| {
        input.set_direction(Direction::In)?;
        input.set_edge(Edge::FallingEdge)?;
        let mut poller = input.get_poller()?;
        loop {
            // needs debounce logic!
            match poller.poll(1000)? {
                Some(_value) => client.press(button_code).call().unwrap(),
                None => "none".to_string(),
            };
        }
    })
}

// spawn a thread to create an interrupt on a single pin
fn spawn_interrupt(pin: u64, button_code: u8) {
    thread::spawn(move || match interrupt(pin, button_code) {
        Ok(()) => println!("Interrupting Complete!"),
        Err(err) => println!("Error: {}", err),
    });
}

fn main() {
    // center joystick
    spawn_interrupt(462, 0);
    // left joystick
    spawn_interrupt(485, 1);
    // right joystick
    spawn_interrupt(481, 2);
    // up joystick
    spawn_interrupt(475, 3);
    // down joystick
    spawn_interrupt(480, 4);
    // A `#5`
    spawn_interrupt(463, 5);
    // B `#6`
    spawn_interrupt(464, 6);
}
