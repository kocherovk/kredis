use std::collections::HashMap;
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::{TcpListener, TcpStream};
use std::{str, thread};
use std::any::Any;

type Key = u8;
type Val = u8;

use std::mem;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
enum Command {
    Set { key: Key, val: Val},
    Get { key: Key }
}

pub trait Into<Command>: Sized {
    fn into(self) -> Command;
}

#[derive(Debug)]
enum CommandCode {
    Get,
    Set
}

#[derive(Debug)]
struct UnknownCommandCode;

impl TryInto<CommandCode> for u8 {
    type Error = UnknownCommandCode;

    fn try_into(self) -> Result<CommandCode, UnknownCommandCode> {
        match self {
            b'g' => Ok(CommandCode::Get),
            b's' => Ok(CommandCode::Set),
            _ => Err(UnknownCommandCode)
        }
    }
}

fn read_get_command(raw: &[u8]) -> Command {
    Command::Get {key: raw[0]}
}

fn read_set_command(raw: &[u8]) -> Command {
    Command::Set {key: raw[0], val: raw[1]}
}

fn get_value(store: &HashMap<Key, Val>, key: Key) -> Option<Val> {
    println!("Getting value {:?}", key);
    store.get(&key).copied()
}

fn set_value(store: &mut HashMap<Key, Val>, key: Key, val: Val) {
    println!("Setting value {:?} to {:?}", key, val);
    store.insert(key, val);
}

fn execute(store: &Mutex<HashMap<Key, Val>>, command: Command) -> Option<u8> {
    let mut state = store.lock().unwrap();
    match command {
        Command::Get { key } => get_value(&state, key),
        Command::Set { key, val } => { set_value(&mut state, key, val); None }
    }
}

fn handle_client(store: &Mutex<HashMap<Key, Val>>, mut stream: TcpStream) {
    let mut command_code_buffer = [0];

    loop {
        let size = stream.read_exact(&mut command_code_buffer);
        println!("read {:?} bytes", size);

        let command_code: CommandCode = command_code_buffer[0].try_into().unwrap();
        println!("command code {:?}", command_code);

        // todo read Err(Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }) bytes
        let command = match command_code {
            CommandCode::Get => {
                // todo it would be better to calculate size of that buffer based on the command type
                let mut get_command_buffer = [0; 1];
                let res = stream.read_exact(&mut get_command_buffer);
                read_get_command(&get_command_buffer)
            }
            CommandCode::Set => {
                let mut set_command_buffer = [0; 2];
                let res = stream.read_exact(&mut set_command_buffer);
                read_set_command(&set_command_buffer)
            }
        };

        println!("command {:?}", command);

        let result = execute(store, command);

        if let Some(res) = result {
            let res = [res];
            let result = stream.write(&res);
            result.unwrap();
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1724")?;
    let global_store = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let store = Arc::clone(&global_store);
        let client_address = stream.peer_addr().unwrap().to_string();
        println!("accepted new client {}", client_address);

        thread::Builder::new()
            .name(String::from("client_thread"))
            .spawn(move || {
                handle_client(&store, stream);
            }).unwrap();
    }
    Ok(())
}