cargo build --release || exit 1 

echo "rust byte counter!: "
time zcat ./test.log.gz | ./target/release/log-parser

echo "js byte counter:"

time zcat ./test.log.gz | node log-parser.js



