use log::{debug, warn};
use radis_lib::io::Command;
use std::io::BufReader;
use std::io::{BufRead, Read};



#[derive(Debug)]
pub enum InvalidCommand {
    EmptyCommand,
    UnknownCommand,
    WrongNumberOfArgs
}

#[derive(Debug)]
pub enum CommandCode {
    Get,
    Set,
}

const MAX_SYMBOLS_PER_COMMAND: u64 = 1024;

impl TryInto<CommandCode> for &str {
    type Error = InvalidCommand;

    fn try_into(self) -> Result<CommandCode, InvalidCommand> {
        match self {
            "get" => Ok(CommandCode::Get),
            "set" => Ok(CommandCode::Set),
            _ => Err(InvalidCommand::UnknownCommand),
        }
    }
}

pub struct PlainTextReader<R> {
    reader: BufReader<R>,
}

impl<R: Read> PlainTextReader<R> {
    pub fn new(stream: R) -> PlainTextReader<R> {
        PlainTextReader {
            // reader: BufReader::new(stream.take(MAX_SYMBOLS_PER_COMMAND)),
            reader: BufReader::new(stream),
        }
    }
}

fn parse_command(line: &str) -> Result<Command, InvalidCommand> {
    let mut command_string = line.trim().split(' ');
    let command_name = match command_string.next() {
        Some(code) => code,
        None => { return Err(InvalidCommand::EmptyCommand) }
    };

    let command_code = command_name.try_into()?;
    debug!("command code {:?}", command_code);

    let mut command_string = command_string.map(| arg | Vec::from(arg.as_bytes()) );

    let command = match command_code {
        CommandCode::Get => {
            let key = command_string.next().ok_or(InvalidCommand::WrongNumberOfArgs)?;
            if command_string.next().is_some() {
                return Err(InvalidCommand::WrongNumberOfArgs)
            }
            Command::Get { key }
        }
        CommandCode::Set => {
            let key = command_string.next().ok_or(InvalidCommand::WrongNumberOfArgs)?;
            let val = command_string.next().ok_or(InvalidCommand::WrongNumberOfArgs)?;
            if command_string.next().is_some() {
                return Err(InvalidCommand::WrongNumberOfArgs)
            }
            Command::Set { key, val }
        }
    };

    return Ok(command);
}


impl<S: Read> Iterator for PlainTextReader<S> {
    type Item = Result<Command, InvalidCommand>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut line = String::new();

        let read_result = self.reader.read_line(&mut line);
        if let Ok(0) = read_result {
            return None
        }
        if let Err(error) = read_result {
            warn!("error reading command: {:?}", error);
            return None
        }

        return Some(parse_command(&line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radis_lib::io::Command::{Get, Set};

    #[test]
    fn read_get() {
        let input = "get 1".as_bytes();
        let mut reader = PlainTextReader::new(input);
        let command = reader.next().unwrap().unwrap();
        assert_eq!(command, Get { key: "1".into() })
    }

    #[test]
    fn read_set() {
        let input = "set 2 3".as_bytes();
        let mut reader = PlainTextReader::new(input);
        let command = reader.next().unwrap().unwrap();
        assert_eq!(
            command,
            Set {
                key: "2".into(),
                val: "3".into()
            }
        )
    }

    #[test]
    fn read_multiple_commands() {
        let input = "set 4 5\nget 6".as_bytes();
        let mut reader = PlainTextReader::new(input);
        let command = reader.next().unwrap().unwrap();
        assert_eq!(
            command,
            Set {
                key: "4".into(),
                val: "5".into()
            }
        );
        let command = reader.next().unwrap().unwrap();
        assert_eq!(command, Get { key: "6".into() });
    }
}
