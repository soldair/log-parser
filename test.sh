cargo fmt
cargo build 1>&2 && cat example.log | RUST_BACKTRACE=1 ./target/debug/log-parser 
