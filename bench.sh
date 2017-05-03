cargo build --release || exit 1 


echo "rust byte counter!: "
start=$(($(date +%s%N)/1000000))
cat ./test.log | ./target/release/log-parser 1> /dev/null
end=$(($(date +%s%N)/1000000))
rust_result=$((end-start))


echo "js byte counter:"

start=$(($(date +%s%N)/1000000))
cat ./test.log | node log-parser.js 1> /dev/null
end=$(($(date +%s%N)/1000000))
js_result=$((end-start))


echo "rust time:"
echo "  "$rust_result

echo "js time:"
echo "  "$js_result


echo "js time - rust time:"
echo "  "$((js_result-rust_result))
