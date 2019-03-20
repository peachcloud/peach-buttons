// https://stackoverflow.com/questions/33626480/raspberry-pi-gpio-pull-up-down-resistors-with-sysfs

extern crate sysfs_gpio;

use std::{thread, time};
use sysfs_gpio::{Direction, Edge, Pin};

fn interrupt(pin: u64, button: String) -> sysfs_gpio::Result<()> {
    //let eighty_millis = time::Duration::from_millis(100);
    let input = Pin::new(pin);
    input.with_exported(|| {
        //thread::sleep(eighty_millis);
        input.set_direction(Direction::In)?;
        input.set_edge(Edge::FallingEdge)?;
        let mut poller = input.get_poller()?;
        loop {
            match poller.poll(1000)? {
                Some(value) => println!("{}", button),
                None => { }
            }
        }
    })
}

fn main() {
    println!("{}", "Launched poller");
    
    thread::spawn(move || {
        let button : String = "left".to_string();
        match interrupt(485, button) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
    
    thread::spawn(move || {
        let button : String = "right".to_string();
        match interrupt(481, button) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
    
    thread::spawn(move || {
        let button : String = "up".to_string();
        match interrupt(475, button) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
    
    thread::spawn(move || {
        let button : String = "down".to_string();
        match interrupt(480, button) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
    
    thread::spawn(move || {
        let button : String = "center".to_string();
        match interrupt(462, button) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });
        
    thread::spawn(move || {
        let button : String = "A".to_string();
        match interrupt(463, button) {
            Ok(()) => println!("Interrupting Complete!"),
            Err(err) => println!("Error: {}", err),
        }
    });

    let button : String = "B".to_string();
    match interrupt(464, button) {
        Ok(()) => println!("Interrupting Complete!"),
        Err(err) => println!("Error: {}", err),
    }
}
