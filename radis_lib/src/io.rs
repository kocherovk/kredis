pub type Key = Vec<u8>;
pub type Val = Vec<u8>;

#[derive(Debug, PartialEq)]
pub enum Command {
    Set { key: Key, val: Val },
    Get { key: Key },
}
