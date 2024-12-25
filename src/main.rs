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
        "ssh" => parse_ssh(args),
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
            print!(r#"{{"time": "{time}", "src_raw": "{src}", "dst_raw": "{dst}""#);

            print_host_details("src", src);
            print_host_details("dst", dst);

            println!(r#"}}"#);
        },
        None => {
            eprintln!("Could not parse line: {line}");
        }
    }
}

fn print_host_details(name: &str, host: &str) {
    if let Some((host, port)) = split_host_and_port(&host) {
        print!(r#", "{name}_host": "{host}", "{name}_port": "{port}""#);

        if let Some(group) = parse_host_group(host) {
            print!(r#", "{name}_group": "{group}""#);
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

fn parse_host_group(host: &str) -> Option<&str> {
    let Some(last_char) = host.chars().last() else { return None; };

    // Can't be a hostname if the last char is a number
    if last_char.is_ascii_digit() { return Some(host); }

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

fn parse_ssh(_args: Args) {
    // Parse failed password login attempts from sshd logs
    // journalctl -u ssh.service > ssh.log
    // Example lines:
    // Dec 25 20:16:35 <hostname> sshd[992]: Invalid user opc from 137.184.84.118 port 58260
    // Dec 25 20:16:43 <hostname> sshd[994]: Failed password for root from 137.184.84.118 port 38640 ssh2
    let re = Regex::new(r"(?xi)
        ^
        (?P<time>\w\w\w\s\d\d\s\d\d:\d\d:\d\d)
        \s
        (?P<hostname>[^\s]+)
        \ssshd\[
        (?P<pid>\d+)
        \]:\s
        (?:failed\spassword\sfor|invalid\suser)
        \s
        (?P<username>[^\s]+)
        \sfrom\s
        (?P<ip>[^\s]+)
        \sport\s
        (?<port>\d+)
    ").unwrap();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                parse_ssh_line(line.as_str(), &re);
            },
            Err(_e) => {
                eprintln!("Could not read line from stdin");
            }
        }
    }
}

fn parse_ssh_line(line: &str, re: &Regex) {
    match re.captures(&line) {
        Some(caps) => {
            let time = &caps[1];
            let hostname = &caps[2];
            let pid = &caps[3];
            let username = &caps[4];
            let ip = &caps[5];
            let port = &caps[6];

            println!(r#"{{"time": "{time}", "hostname": "{hostname}", "pid": "{pid}", "username": "{username}", "ip": "{ip}", "port": "{port}"}}"#);
        },
        None => {
            // Do nothing
            // Unless there's some kind of debug flag enabled
            // eprintln!("Could not parse line: {line}");
        }
    }
}
