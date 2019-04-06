extern crate sysfs_gpio;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;

use std::thread;
use sysfs_gpio::{Direction, Edge, Pin};
use jsonrpc_client_http::HttpTransport;

// json-rpc client
jsonrpc_client!(pub struct PeachMenuClient {
    // send button code to peach-menu
    pub fn press(&mut self, button_code: u8) -> RpcRequest<String>;
});

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
            match poller.poll(1000)? {
                Some(_value) => {
                    client.press(button_code).call().unwrap()
                },
                None => "none".to_string()
            };
        }
    })
}

fn main() {

    thread::spawn(move || {
        // center joystick
        let button_code : u8 = 0;
        match interrupt(462, button_code) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
    
    thread::spawn(move || {
        // left joystick
        let button_code : u8 = 1;
        match interrupt(485, button_code) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
    
    thread::spawn(move || {
        // right joystick
        let button_code : u8 = 2;
        match interrupt(481, button_code) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
    
    thread::spawn(move || {
        // up joystick
        let button_code : u8 = 3;
        match interrupt(475, button_code) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
    
    thread::spawn(move || {
        // down joystick
        let button_code : u8 = 4;
        match interrupt(480, button_code) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
    
    thread::spawn(move || {
        // A button (#5)
        let button_code : u8 = 5;
        match interrupt(463, button_code) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });

    // B button (#6)
    let button_code : u8 = 6;
    match interrupt(464, button_code) {
        Ok(()) => println!("Interrupting Complete!"),
        Err(err) => println!("Error: {}", err),
    }
}
