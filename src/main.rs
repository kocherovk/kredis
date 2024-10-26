use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::ops::Deref;
use std::thread;

use log::{debug, info};
use radis_impl::reader::plaintext::PlainTextReader;
use radis_impl::storage::hashmap::HashMapStorage;
use radis_lib::io::{Command, Key, Val};
use radis_lib::storage::Storage;
use std::sync::Arc;

fn get_value(store: &dyn Storage<Key, Val>, key: Key) -> Option<Val> {
    debug!("Getting value {:?}", key);
    store.get(&key)
}

fn set_value(store: &dyn Storage<Key, Val>, key: Key, val: Val) {
    debug!("Setting value {:?} to {:?}", key, val);
    store.set(key, val);
}

fn execute(store: &dyn Storage<Key, Val>, command: Command) -> Option<Val> {
    match command {
        Command::Get { key } => get_value(store, key),
        Command::Set { key, val } => {
            set_value(store, key, val);
            None
        }
    }
}

fn main() -> std::io::Result<()> {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:1721")?;
    let global_state = Arc::new(HashMapStorage::<Key, Val>::new());

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let store = Arc::clone(&global_state);
        let client_address = stream.peer_addr().unwrap().to_string();
        info!("accepted new client {}", client_address);

        thread::Builder::new()
            .name(String::from("client_thread"))
            .spawn(move || {
                handle_client(store.deref(), stream);
            })
            .unwrap();
    }
    Ok(())
}

fn handle_client(store: &dyn Storage<Key, Val>, mut stream: TcpStream) {
    let reader = PlainTextReader::new(stream.try_clone().unwrap());

    for command_result in reader {
        let command = match command_result {
            Ok(cmd) => cmd,
            Err(err) => {
                debug!("error reading command {:?}", err);
                continue;
            }
        };

        debug!("command {:?}", command);

        let result = execute(store, command);

        if let Some(res) = result {
            stream.write(&res).unwrap();
            stream.write(b"\n").unwrap();
        }
    }

    debug!("no more commands from client");
}
