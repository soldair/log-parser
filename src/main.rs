use std::io;
use std::io::prelude::*;
//use std::fs::File;

fn main() {

    // <134>2017-04-24T20:38:26Z cache-iad2645 fastly-logs-5-east[367774]: 54.87.185.35 "-" "GET
    // /npm/public/registry/i/is-utf8/_attachments/doc.json" 200 "npm/4.2.0 node/v7.8.0 linux x64"
    // "install" "26f82920d62b24e9" "HIT" "(null)" "cache-iad2645-IAD" "0" "1016" "1466" "251"
    
    // only quoted values have spaces.
    let mut inquote = false;
    let mut value = String::new();
    let mut values = Vec::new();

    for byte in io::stdin().bytes() {
        let c = byte.unwrap() as char;

        if c == '"' {
            if inquote {
                inquote = false;
            } else {
                inquote = true;
            }
            continue;
        }
        

        if c == '\n' {
            for v in values {
                print!("{}, ", v);
            }

            println!("");

            values = Vec::new();//with_capacity(values.len());
        } else if !inquote && c == ' '  {
            // not in a quoted value and i found a space.
            values.push(value);
            value = String::new()
        } else {
            value.push(c);
        }
    }

}

/*
fn format(slice: &str) {
    
}
*/

