nc 127.0.0.1 1721stty -icanon | nc 127.0.0.1 1721

# test
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="target/profraw/radis-%p-%m.profraw"
cargo build
cargo test
grcov target/profraw/ --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing  -o target/coverage

# todo
make library for the core functionality and move it to separate package 
link library to the executable and provide logging library
