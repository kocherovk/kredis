use log::{debug, warn};
use radis_lib::io::Command;
use std::io::BufReader;
use std::io::{BufRead, Read};

#[derive(Debug)]
pub enum InvalidCommand {
    EmptyCommand,
    UnknownCommand,
    WrongNumberOfArgs,
    ReadError,
}

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
            reader: BufReader::new(stream),
        }
    }
}

fn build_command<'a, I>(code: CommandCode, arguments: I) -> Result<Command, InvalidCommand>
where
    I: Iterator<Item = &'a str>,
{
    let mut args = arguments.map(|arg| Vec::from(arg.as_bytes()));
    match code {
        CommandCode::Get => {
            let args = [args.next(), args.next()];
            match args {
                [Some(key), None] => Ok(Command::Get { key }),
                _ => Err(InvalidCommand::WrongNumberOfArgs),
            }
        }
        CommandCode::Set => {
            let args = [args.next(), args.next(), args.next()];
            match args {
                [Some(key), Some(val), None] => Ok(Command::Set { key, val }),
                _ => Err(InvalidCommand::WrongNumberOfArgs),
            }
        }
    }
}

fn parse_command(line: &str) -> Result<Command, InvalidCommand> {
    let mut command_string = line.trim().split_whitespace();
    let command_name = match command_string.next() {
        Some(code) => code,
        None => return Err(InvalidCommand::EmptyCommand),
    };
    build_command(command_name.try_into()?, command_string)
}

impl<S: Read> Iterator for PlainTextReader<S> {
    type Item = Result<Command, InvalidCommand>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut line = String::new();
        let read_result = self.reader.read_line(&mut line);

        if let Ok(0) = read_result {
            return None;
        }

        if let Err(error) = read_result {
            warn!("error reading command: {:?}", error);
            return Some(Err(InvalidCommand::ReadError));
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
