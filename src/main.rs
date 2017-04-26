use std::io;
use std::io::prelude::*;
use std::fs::File;

extern crate time;


fn main() {

    // <134>2017-04-24T20:38:26Z cache-iad2645 fastly-logs-5-east[367774]: 54.87.185.35 "-" "GET
    // /npm/public/registry/i/is-utf8/_attachments/doc.json" 200 "npm/4.2.0 node/v7.8.0 linux x64"
    // "install" "26f82920d62b24e9" "HIT" "(null)" "cache-iad2645-IAD" "0" "1016" "1466" "251"

    let stdin = io::stdin();
    let handle = stdin.lock();
    let mut count = 0; 

    let mut f = File::open("./test.log").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();

    println!("{}", buffer.len());
}


fn format(values: &Vec<String>) {
    let len = values.len();

    if len == 0 {
        return;
    }

    let date = parse_date(values[0].clone());
    let unixtime = logtime_to_unixtime(&date);
    let minute = unixtime - (unixtime % 60);

    // invalid time. wont count.
    if unixtime == 0 {
        return;
    }

    let pop = substr(&values[1], 6, 3);

    // if i have any values emit them.

    let service = path_to_service(&values[5]);

    let status = &values[6];

    let mut offset = 0;
    if len == 18 {
        offset = 1;
    }

    let egress_bytes = &values[15 + offset];
    let ingress_bytes = &values[16 + offset];

    for v in values {
        print!("{}, ", v);
    }

    println!("[{}]", len);

    println!("format {} {} {} {} {} {} {} {}",
             date,
             minute,
             unixtime,
             pop,
             service,
             status,
             egress_bytes,
             ingress_bytes);
}

fn parse_date(dateref: String) -> String {
    let mut date = dateref.clone();
    let offset_option = date.find('>');

    if offset_option != None {

        let offset = offset_option.unwrap() + 1;
        // -1 to remove the trailing Z
        let end = date.len() - offset - 1;

        //date = date.chars().skip(offset).take(end).collect();
        date = substr(&date, offset, end)
    }
    return date;
}

fn logtime_to_unixtime(date: &String) -> i64 {

    // 2017-04-24T20:38:26
    let unixtime = match time::strptime(date, "%Y-%m-%dT%H:%M:%S") {
        Ok(v) => v.to_timespec().sec,
        _ => 0,
    };

    return unixtime;
}

fn substr(string: &String, start: usize, len: usize) -> String {
    let copy = string.clone();
    return copy.chars().skip(start).take(len).collect();
}


fn path_to_service(path: &String) -> &'static str {

    if path.contains('@') {
        if path.contains(".tgz") {
            return "scoped-tarball";
        }
        return "scoped-json";
    }

    if path.contains("doc.json") {
        return "static-json";
    }

    if path.contains("doc.min.json") {
        return "corgi-json";
    }

    let len = path.len();

    if substr(&path, len - 4, 5) == ".tgz" {
        return "tarball";
    }

    if path.contains("/-/") {
        return "tie-fighter";
    }

    if !split_at(&path, ' ').contains('/') {
        return "json";
    }
    return "misc";
}

fn split_at(string: &String, find: char) -> String {
    return substr(&string, string.find(find).unwrap_or(0), string.len());
}
