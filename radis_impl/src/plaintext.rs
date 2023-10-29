use log::warn;
use radis_lib::io::Command;
use std::io::BufReader;
use std::io::{BufRead, Read};

#[derive(Debug, PartialEq)]
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
        let input = "get key";
        let mut commands = PlainTextReader::new(input.as_bytes());
        assert_eq!(commands.next(), Some(Ok(Get { key: "key".into() })));
    }

    #[test]
    fn read_from_empty_stream() {
        let input = "";
        let mut commands = PlainTextReader::new(input.as_bytes());
        assert_eq!(commands.next(), None);
    }

    #[test]
    fn read_set() {
        let input = "set 1 2";
        let mut commands = PlainTextReader::new(input.as_bytes());
        assert_eq!(
            commands.next(),
            Some(Ok(Set { key: "1".into() , val: "2".into()}))
        );
    }

    #[test]
    fn read_unknown_command() {
        let input = "unknown 1 2";
        let mut commands = PlainTextReader::new(input.as_bytes());
        assert_eq!(
            commands.next(),
            Some(Err(InvalidCommand::UnknownCommand))
        );
    }

    #[test]
    fn test_get_command_wrong_number_of_args() {
        let input = "get 1 2\nget";
        let mut commands = PlainTextReader::new(input.as_bytes());
        assert_eq!(commands.next(),Some(Err(InvalidCommand::WrongNumberOfArgs)));
        assert_eq!(commands.next(),Some(Err(InvalidCommand::WrongNumberOfArgs)));
    }

    #[test]
    fn test_set_command_wrong_number_of_args() {
        let input = "set\nset 1\nset 1 2 3";
        let mut commands = PlainTextReader::new(input.as_bytes());
        assert_eq!(commands.next(),Some(Err(InvalidCommand::WrongNumberOfArgs)));
        assert_eq!(commands.next(),Some(Err(InvalidCommand::WrongNumberOfArgs)));
        assert_eq!(commands.next(),Some(Err(InvalidCommand::WrongNumberOfArgs)));
    }

    #[test]
    fn read_empty_command() {
        let input = "\n";
        let mut commands = PlainTextReader::new(input.as_bytes());
        assert_eq!(commands.next(),Some(Err(InvalidCommand::EmptyCommand)));
    }

    #[test]
    fn read_multiple_commands() {
        let input = "set 4 5\nget 6";
        let mut commands = PlainTextReader::new(input.as_bytes());
        assert_eq!(
            commands.next(),
            Some(Ok(Set {key: "4".into(), val: "5".into()}))
        );
        assert_eq!(
            commands.next(),
            Some(Ok(Get { key: "6".into() }))
        );
    }
}
