cargo build 1>&2
cat example.log | ./target/debug/log-parser 
