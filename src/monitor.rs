use sysinfo::{NetworkExt, ProcessorExt, SystemExt};
use serde_derive::{Serialize, Deserialize};
use std::vec::Vec;
use std::sync::mpsc::channel;
use std::{thread, time};
use websocket::{Message, OwnedMessage};
use websocket::client::ClientBuilder;

use crate::configuration::{Configuration};
#[derive(Serialize, Deserialize)]
pub struct NetStatInfo {
    interface_name: String,
    income_bytes: u64,
    outcome_bytes: u64
}

#[derive(Serialize, Deserialize)]
pub struct StatInfo {
    total_memory: u64,
    used_memory: u64,
    processors: Vec<f32>,
    net_interfaces: Vec<NetStatInfo>
}

pub fn start(configuration: &mut Configuration) {
    let mut system = sysinfo::System::new_all();
    let client = ClientBuilder::new(&configuration.monitor_server_url)
        .unwrap()
        .add_protocol("rust-websocket")
        .connect_insecure()
        .unwrap();
    
    let (mut receiver, mut sender) = client.split().unwrap();
    let (tx, rx) = channel();

    let tx_1 = tx.clone();
    
    let _send_loop = thread::spawn(move || {
		loop {
			// Send loop
			let message = match rx.recv() {
				Ok(m) => m,
				Err(e) => {
					println!("Send Loop: {:?}", e);
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					let _ = sender.send_message(&message);
					// If it's a close message, just send it and then return.
					return;
				}
				_ => (),
			}
			// Send the message
			match sender.send_message(&message) {
				Ok(()) => (),
				Err(e) => {
					println!("Send Loop: {:?}", e);
					let _ = sender.send_message(&Message::close());
					return;
				}
			}
		}
	});

	let _receive_loop = thread::spawn(move || {
		// Receive loop
		for message in receiver.incoming_messages() {
			let message = match message {
				Ok(m) => m,
				Err(e) => {
					println!("Receive Loop: {:?}", e);
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					// Got a close message, so send a close message and return
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
				OwnedMessage::Ping(data) => {
					match tx_1.send(OwnedMessage::Pong(data)) {
						// Send a pong in response
						Ok(()) => (),
						Err(e) => {
							println!("Receive Loop: {:?}", e);
							return;
						}
					}
				}
				// Say what we received
				_ => println!("Receive Loop: {:?}", message),
			}
		}
	});

    loop {
        system.refresh_all();
        let mut processors_stats = Vec::<f32>::new();
        let mut net_interfaces = Vec::<NetStatInfo>::new();

        for processor in system.get_processors() { 
            processors_stats.push(processor.get_cpu_usage());
        }

        for (interface_name, net_interface) in system.get_networks() { 
            net_interfaces.push(NetStatInfo {
                interface_name: interface_name.to_owned(),
                income_bytes: net_interface.get_income(),
                outcome_bytes: net_interface.get_outcome()
            });
        }
        let stat_info = StatInfo {
            total_memory: system.get_total_memory(),
            used_memory: system.get_used_memory(),
            processors: processors_stats,
            net_interfaces: net_interfaces
        };
		let trimmed = serde_json::to_string_pretty(&stat_info).unwrap();
        let trimmed : &str =  &trimmed[..];
		let message = match trimmed {
			"/close" => {
				// Close the connection
				let _ = tx.send(OwnedMessage::Close(None));
				break;
			}   
			// Send a ping
			"/ping" => OwnedMessage::Ping(b"PING".to_vec()),
			// Otherwise, just send text
			_ => OwnedMessage::Text(trimmed.to_string()),
		};

		match tx.send(message) {
			Ok(()) => (),
			Err(e) => {
				println!("Main Loop: {:?}", e);
				break;
			}
        }
        thread::sleep(time::Duration::from_millis(configuration.refresh_time_millis));
	}
}