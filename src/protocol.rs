use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;

pub const MAGIC_BYTES: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

pub const OFFLINE_PING: u8 = 0x01;

pub struct OfflinePingPacket {
    pub start_time: u64,
    pub client_id: u64,
}

pub struct OfflinePongPacket {
    pub response_time: u64,
    pub server_id: u64,
    pub motd: String,
}

impl OfflinePongPacket {
    pub fn decode(buffer: Vec<u8>) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        cursor.read_u8(); // packet id
        let response_time = cursor.read_u64::<BigEndian>();
        let server_id = cursor.read_u64::<BigEndian>();
        cursor.set_position(cursor.position() + MAGIC_BYTES.len() as u64);
        let motd_len = cursor.read_u16::<BigEndian>();
        if motd_len.is_err() || server_id.is_err() || response_time.is_err() {
            return None;
        }
        let response_time = response_time.unwrap();
        let server_id = server_id.unwrap();
        let motd_len = motd_len.unwrap();
        let mut motd = Vec::new();
        motd.resize(motd_len as usize, 0);
        cursor.read(&mut motd);
        let motd: String = String::from_utf8(motd).unwrap();
        Some(OfflinePongPacket {
            response_time,
            server_id,
            motd,
        })
    }
}

impl OfflinePingPacket {
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::<u8>::new();
        packet.push(OFFLINE_PING);
        packet.write_u64::<BigEndian>(self.start_time);
        packet.write(&MAGIC_BYTES);
        packet.write_u64::<BigEndian>(self.client_id);
        packet
    }
}
