use std::env;
use std::env::Args;
use std::io::{self, BufRead};
use regex::Regex;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let mut args = env::args();

    // Skip binary name
    args.next();

    // Get command name
    match args.next() {
        None => show_usage(),
        Some(command) => process_command(command, args)
    }
}

fn show_usage() {
    eprintln!("{NAME} {VERSION}");
    eprintln!("Usage");
    eprintln!("    {NAME} tcpdump    Convert tcpdump output to json");
}

fn process_command(command: String, args: Args) {
    match command.as_str() {
        "tcpdump" => parse_tcpdump(args),
        _ => unknown_command(&command),
    }
}

fn unknown_command(command: &str) {
    eprintln!("Unknown command: {command}");
    show_usage();
}

fn parse_tcpdump(_args: Args) {
    let stdin = io::stdin();
    let re = Regex::new(r"(?x)
        ^
        (?P<time>[^\s]+)
        \sIP\s
        (?P<src>[^\s]+) # <ip>.<port> or <hostname>.<port>
        \s>\s
        (?P<dst>[^:]+) # <ip>.<port> or <hostname>.<port>
    ").unwrap();

    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                parse_tcpdump_line(line.as_str(), &re);
            },
            Err(_e) => {
                eprintln!("Could not read line from stdin");
            }
        }
    }
}

fn parse_tcpdump_line(line: &str, re: &Regex) {
    match re.captures(&line) {
        Some(caps) => {
            let time = &caps[1];
            let src = &caps[2];
            let dst = &caps[3];
            print!(r#"{{"time": "{time}", "src.raw": "{src}", "dst.raw": "{dst}""#);

            if let Some((host, port)) = split_host_and_port(&src) {
                print!(r#", "src.host": "{host}", "src.port": "{port}""#);

                if let Some(apex) = parse_apex_domain(host) {
                    print!(r#", "src.apex": "{apex}""#);
                }
            }

            if let Some((host, port)) = split_host_and_port(&dst) {
                print!(r#", "dst.host": "{host}", "dst.port": "{port}""#);

                if let Some(apex) = parse_apex_domain(host) {
                    print!(r#", "dst.apex": "{apex}""#);
                }
            }

            println!(r#"}}"#);
        },
        None => {
            eprintln!("Could not parse line: {line}");
        }
    }
}

fn split_host_and_port(combined: &str) -> Option<(&str, &str)> {
    match combined.rfind(".") {
        Some(position) => {
            return Some((&combined[..position], &combined[position + 1..]));
        },
        None => {
            return None;
        }
    };
}

fn parse_apex_domain(host: &str) -> Option<&str> {
    let Some(last_char) = host.chars().last() else { return None; };

    // Can't be a hostname if the last char is a number
    if last_char.is_ascii_digit() { return None; } 

    // Ok, so it's probably a hostname.
    // First find where the tld starts
    let Some(tld_start) = host.rfind(".") else { return None; };

    // Then, find where the apex starts. There may or may not be more to it
    match host[..tld_start].rfind(".") {
        Some(apex_start) => {
            return Some(&host[apex_start + 1..]);
        },
        None => {
            return Some(&host);
        }
    }
}
