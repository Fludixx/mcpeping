use crate::protocol::{OfflinePingPacket, OfflinePongPacket};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use clap::{App, Arg};
use colorful::core::color_string::CString;
use colorful::Colorful;
use rand::{random, thread_rng, Rng};
use std::io::{Cursor, Read, Write};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::process::exit;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, SystemTime};

mod protocol;

fn parse_address(addr: &str) -> Option<SocketAddr> {
    let address = SocketAddr::from_str(addr);
    if address.is_err() {
        // check if is domain
        let mut domain_addr;
        if !addr.contains(":") {
            domain_addr = String::from(addr) + ":19132";
        } else {
            domain_addr = String::from(addr);
        }
        let socket_addrs = domain_addr.to_socket_addrs();
        if socket_addrs.is_ok() {
            let socket_addrs = socket_addrs.unwrap();
            for addr in socket_addrs {
                return Some(addr);
            }
        }
        // check if user forgot to add port
        let ip = IpAddr::from_str(addr);
        if ip.is_err() {
            return None;
        }
        return Some(SocketAddr::new(ip.unwrap(), 19132));
    }
    Some(address.unwrap())
}

fn main() {
    let matches = App::new("Minecraft: Bedrock Edition Pinger")
        .version(clap::crate_version!())
        .author(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(Arg::with_name("server").help("Server to ping"))
        .arg(Arg::with_name("motd").short("m").long("motd"))
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .takes_value(true)
                .default_value("3"),
        )
        .arg(
            Arg::with_name("loops")
                .short("n")
                .long("loops")
                .takes_value(true)
                .default_value("-1"),
        )
        .get_matches();
    if !matches.is_present("server") {
        println!("No server Provided");
        exit(1);
    }
    let server = parse_address(matches.value_of("server").unwrap());
    let timeout = u32::from_str(matches.value_of("timeout").unwrap());
    if timeout.is_err() {
        println!("Invalid timeout provided.");
        exit(1);
    }
    let timeout = timeout.unwrap();
    if server.is_none() {
        println!("Invalid address provided.");
        exit(1);
    }
    let server = server.unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0");
    if socket.is_err() {
        println!("Failed to bind");
        exit(1);
    }
    let socket = socket.unwrap();
    socket.connect(server);
    let mut looped: i32 = 0;
    let max_loops_arg = matches.value_of("loops").unwrap();
    let max_loops;
    let max_loops_res = i32::from_str(max_loops_arg);
    if max_loops_res.is_err() {
        println!("Invalid loop number");
        exit(1);
    }
    max_loops = max_loops_res.unwrap();
    while looped != max_loops {
        let client_id = random::<u64>();
        socket.set_read_timeout(Some(Duration::from_secs(timeout as u64)));
        let start_time = SystemTime::now();
        let ping_packet = OfflinePingPacket {
            start_time: start_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            client_id,
        };
        socket.send(ping_packet.encode().as_slice());
        let mut buff: Vec<u8> = Vec::new();
        buff.resize(1024, 0);
        let request = socket.recv_from(&mut buff);
        let end_time = SystemTime::now();
        // some servers are sending multiple packets on 1 request?
        socket.set_read_timeout(Some(Duration::from_millis(1)));
        while socket.peek(&mut [0; 1024]).is_ok() {
            socket.recv(&mut [0; 1024]);
        }
        if request.is_err() {
            println!("Timed out.");
        } else {
            let (len, src) = request.unwrap();
            let response = OfflinePongPacket::decode(buff);
            if response.is_none() {
                println!("Invalid response packet from: {}", src.to_string())
            } else {
                let response = response.unwrap();
                let delay = end_time.duration_since(start_time).unwrap().as_millis();
                let delay_str: CString = match delay {
                    d if d < 5 => format!("{}ms", delay).light_green(),
                    d if d < 30 => format!("{}ms", delay).green(),
                    d if d < 60 => format!("{}ms", delay).yellow(),
                    d if d < 100 => format!("{}ms", delay).light_red(),
                    d if d < 500 => format!("{}ms", delay).red(),
                    _ => format!("{}ms", delay).magenta(),
                };
                println!(
                    "{} bytes from {} ({:x}) in: {}",
                    len,
                    src.to_string().light_blue(),
                    response.server_id,
                    delay_str,
                );
                if matches.is_present("motd") {
                    println!("MOTD: {}", response.motd);
                }
            }
        }
        thread::sleep(Duration::from_secs(1));
        looped += 1;
    }
}
