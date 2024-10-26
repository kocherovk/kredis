pub type Key = Vec<u8>;
pub type Val = Vec<u8>;

#[derive(Debug, PartialEq)]
pub enum Command {
    Set { key: Key, val: Val },
    Get { key: Key },
}

#[derive(Debug, PartialEq)]
pub enum InvalidCommand {
    EmptyCommand,
    UnknownCommand,
    WrongNumberOfArgs,
    ReadError,
}

pub trait CommandReader: IntoIterator<Item=Result<Command, InvalidCommand>> {}