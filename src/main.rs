use std::io;
use std::io::prelude::*;
use std::collections::HashMap;

extern crate time;

struct Line {
    date: String,
    minute: i64,
    service: String,
    status: String,
    pop: String,
    bytes: i64,
    duration: i64,
    hit: bool,
}

struct Count {
    count: i64,
    size: i64,
    duration: i64,
    hits: i64,
}

fn main() {

    // <134>2017-04-24T20:38:26Z cache-iad2645 fastly-logs-5-east[367774]: 54.87.185.35 "-" "GET
    // /npm/public/registry/i/is-utf8/_attachments/doc.json" 200 "npm/4.2.0 node/v7.8.0 linux x64"
    // "install" "26f82920d62b24e9" "HIT" "(null)" "cache-iad2645-IAD" "0" "1016" "1466" "251"

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = [0; 1024];
    let mut byte_count = 0;

    let mut state = "out";

    let mut value = String::new();
    let mut values = vec!["".to_string(); 32];
    let mut values_offset: usize = 0;
    let mut line = Line {
        date: String::new(),
        minute: 0,
        pop: String::new(),
        service: String::new(),
        status: String::new(),
        bytes: 0,
        duration: 0,
        hit: false,
    };

    let mut last_n_minutes = vec![];
    let mut minutes: HashMap<i64, HashMap<String, Count>> = HashMap::new();

    loop {
        let count = match handle.read(&mut buf) {
            Ok(v) => v,
            _ => 0,
        };

        if count == 0 {
            break;
        }

        for i in 0..count {
            let c = buf[i] as char;
            byte_count += 1;

            if c == '\n' {
                // if i have a pending value add it to values
                if value.len() > 0 {
                    values[values_offset] = value;
                }

                if values_offset >= 17 {
                    if format(&values, values_offset, &mut line) {
                        if !minutes.contains_key(&line.minute) {
                            // minute of hashmaps.
                            minutes.insert(line.minute, HashMap::new());

                            last_n_minutes.push(line.minute);

                            if last_n_minutes.len() > 5 {
                                last_n_minutes.sort();
                                let complete_minute = last_n_minutes.remove(0);

                                report(complete_minute, &minutes[&complete_minute]);
                                minutes.remove(&complete_minute);
                            }
                        }

                        // add data to current minute!

                        let mut current_minute = minutes.get_mut(&line.minute).unwrap();

                        let mut current_count = current_minute
                            .entry(format!("{}:{}:{}", line.service, line.pop, line.status))
                            .or_insert(Count {
                                           count: 0,
                                           size: 0,
                                           duration: 0,
                                           hits: 0,
                                       });

                        current_count.count += 1;
                        current_count.size += line.bytes;
                        current_count.duration += line.duration;
                        if line.hit {
                            current_count.hits += 1;
                        }
                    }
                } else {
                    println!("should not happen");
                }

                value = String::new();
                values_offset = 0;
                continue;
            }

            if state == "out" {
                if c == '"' {
                    state = "quoted";
                    // skip quote
                    continue;
                } else if c != ' ' {
                    state = "value";
                }
            }

            if state == "value" {
                if c == ' ' {
                    values[values_offset] = value;
                    values_offset += 1;
                    value = String::new();
                    state = "out";
                } else if c != '"' {
                    value.push(c);
                }
            }

            if state == "quoted" {
                if c == '"' {
                    values[values_offset] = value;
                    values_offset += 1;
                    value = String::new();
                    state = "out";
                } else {
                    value.push(c);
                }
            }

        }
    }

    if last_n_minutes.len() > 0 {
        for minute in last_n_minutes {
            report(minute, &minutes[&minute]);
        }
    }

    //io::stderr().write(format!("count {}\n", byte_count).as_bytes());
}


fn format(values: &Vec<String>, length: usize, line: &mut Line) -> bool {

    if length == 0 {
        return false;
    }

    let date = parse_date(&values[0]);
    let unixtime = logtime_to_unixtime(&date);
    let minute = unixtime - (unixtime % 60);
    let minute_date = substr(&date, 0, date.len() - 2) + "00.000Z";

    // invalid time. wont count.
    if unixtime == 0 {
        return false;
    }

    let pop = substr(&values[1], 6, 3);

    // if i have any values emit them.

    let service = path_to_service(&values[5]);
    let status = &values[6];

    let mut offset = 0;
    if length == 18 {
        offset = 1;
    }

    let duration = &values[13].parse::<i64>().unwrap_or(0);
    let egress_bytes = &values[15 + offset].parse::<i64>().unwrap_or(0);
    //let ingress_bytes = &values[16 + offset].parse::<i32>().unwrap_or(0);

    line.date = minute_date;
    line.minute = minute;
    line.service = service.to_string();
    line.pop = pop;
    line.bytes = *egress_bytes;
    line.status = status.to_string();
    line.duration = *duration;
    line.hit = if &values[10] == "HIT" { true } else { false };
    return true;
}

fn report(timestamp: i64, minute: &HashMap<String, Count>) {
    let mut report = format!("{{\"time\":{},[", timestamp);
    let mut i = 0;
    for (name, count) in minute.iter() {
        if i > 0 {
            report.push_str(",")
        }
        i += 1;

        report.push_str(
              &format!("{{\"name\":\"{}\",\"count\":{},\"size\":{},\"duration\":{},\"hits\":{}}}",
              name,
              count.count,
              count.size,
              count.duration,
              count.hits));

    }

    report.push_str("]}}");
    println!("{}", report);
}


fn parse_date(date: &String) -> String {
    let offset_option = date.find('>');

    if offset_option != None {

        let offset = offset_option.unwrap() + 1;
        // -1 to remove the trailing Z
        let end = date.len() - offset - 1;

        //date = date.chars().skip(offset).take(end).collect();
        return substr(&date, offset, end);
    }
    return String::new();
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
    return string.chars().skip(start).take(len).collect();
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
