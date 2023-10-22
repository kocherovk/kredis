use log::debug;
use radis_lib::io::Command;
use std::io::BufReader;
use std::io::{BufRead, Read};

#[derive(Debug)]
pub struct InvalidCommand;

#[derive(Debug)]
pub enum CommandCode {
    Get,
    Set,
}

impl TryInto<CommandCode> for u8 {
    type Error = InvalidCommand;

    fn try_into(self) -> Result<CommandCode, InvalidCommand> {
        match self {
            b'g' => Ok(CommandCode::Get),
            b's' => Ok(CommandCode::Set),
            _ => Err(InvalidCommand),
        }
    }
}

impl TryInto<CommandCode> for &str {
    type Error = InvalidCommand;

    fn try_into(self) -> Result<CommandCode, InvalidCommand> {
        match self {
            "get" => Ok(CommandCode::Get),
            "set" => Ok(CommandCode::Set),
            _ => Err(InvalidCommand),
        }
    }
}

pub fn read_stream(stream: &mut dyn Read) -> Result<Command, InvalidCommand> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let mut command_string = line.trim_end().split(' ');
    let command_code = command_string.next().unwrap().try_into()?;
    debug!("command code {:?}", command_code);

    let command = match command_code {
        CommandCode::Get => {
            let key = command_string.next().unwrap().as_bytes().into();
            Command::Get { key: key }
        }
        CommandCode::Set => {
            let key = command_string.next().unwrap().as_bytes().into();
            let val = command_string.next().unwrap().as_bytes().into();
            Command::Set { key: key, val: val }
        }
    };

    return Ok(command);
}
