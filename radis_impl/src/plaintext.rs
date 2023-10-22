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

pub struct PlainTextReader<R> {
    reader: BufReader<R>,
}

impl<R: Read> PlainTextReader<R> {
    pub fn new(stream: R) -> PlainTextReader<R> {
        PlainTextReader {
            reader: BufReader::new(stream),
        }
    }
}

impl<S: Read> Iterator for PlainTextReader<S> {
    type Item = Result<Command, InvalidCommand>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();

        let mut command_string = line.trim_end().split(' ');
        let command_code = command_string.next().unwrap().try_into().unwrap();
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

        return Some(Ok(command));
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
