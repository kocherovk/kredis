use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use log::{debug, info};
use radis_impl::plaintext::PlainTextReader;
use radis_lib::io::{Command, Key, Val};
use std::sync::{Arc, Mutex};

fn get_value(store: &HashMap<Key, Val>, key: Key) -> Option<Val> {
    debug!("Getting value {:?}", key);
    return store.get(&key).cloned();
}

fn set_value(store: &mut HashMap<Key, Val>, key: Key, val: Val) {
    debug!("Setting value {:?} to {:?}", key, val);
    store.insert(key, val);
}

fn execute(store: &Mutex<HashMap<Key, Val>>, command: Command) -> Option<Val> {
    let mut state = store.lock().unwrap();
    match command {
        Command::Get { key } => get_value(&state, key),
        Command::Set { key, val } => {
            set_value(&mut state, key, val);
            None
        }
    }
}

fn main() -> std::io::Result<()> {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:1721")?;
    let global_store = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let store = Arc::clone(&global_store);
        let client_address = stream.peer_addr().unwrap().to_string();
        info!("accepted new client {}", client_address);

        thread::Builder::new()
            .name(String::from("client_thread"))
            .spawn(move || {
                handle_client(&store, stream);
            })
            .unwrap();
    }
    Ok(())
}

fn handle_client(store: &Mutex<HashMap<Key, Val>>, mut stream: TcpStream) {
    let reader = PlainTextReader::new(stream.try_clone().unwrap());

    for command in reader {
        debug!("command {:?}", command);
        let result = execute(store, command.unwrap());

        if let Some(res) = result {
            stream.write(&res).unwrap();
            stream.write(b"\n").unwrap();
        }
    }
}
