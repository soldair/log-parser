cargo build --release || exit 1 

echo "rust byte counter!: "
time cat ./test.log | ./target/release/log-parser

echo "js byte counter:"

time cat ./test.log | node log-parser.js



