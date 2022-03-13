use std::io::Result;
use crate::reader::{Reader, Endian};
use crate::writer::{Writer};

#[warn(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PacketID {
    UnconnectedPing1 = 0x01,
    UnconnectedPing2 = 0x02,
    UnconnectedPong = 0x1c,
    OpenConnectionRequest1 = 0x05,
    OpenConnectionReply1 = 0x06,
    OpenConnectionRequest2 = 0x07,
    OpenConnectionReply2 = 0x08,
    IncompatibleProtocolVersion = 0x19,
    Unknown = 0xff
}

pub fn transaction_packet_id(id : u8) -> PacketID {
    match id{
        0x01 => PacketID::UnconnectedPing1,
        0x02 => PacketID::UnconnectedPing2,
        0x1c => PacketID::UnconnectedPong,
        0x05 => PacketID::OpenConnectionRequest1,
        0x06 => PacketID::OpenConnectionReply1,
        0x07 => PacketID::OpenConnectionRequest2,
        0x08 => PacketID::OpenConnectionReply2,
        0x19 => PacketID::IncompatibleProtocolVersion,
        _ => PacketID::Unknown
    }
}

pub fn transaction_packet_id_to_u8(packetid : PacketID) -> u8 {
    match packetid{
        PacketID::UnconnectedPing1 => 0x01,
        PacketID::UnconnectedPing2 => 0x02,
        PacketID::UnconnectedPong => 0x1c,
        PacketID::OpenConnectionRequest1 => 0x05,
        PacketID::OpenConnectionReply1 => 0x06,
        PacketID::OpenConnectionRequest2 => 0x07,
        PacketID::OpenConnectionReply2 => 0x08,
        PacketID::IncompatibleProtocolVersion => 0x19,
        PacketID::Unknown => 0xff,
    }
}

macro_rules! unwrap_or_return {
    ($res:expr) => {
        match $res.await {
            Ok(val) => val,
            Err(e) => {
                return Err(e);
            }
        }
    };
}

#[derive(Clone)]
pub struct PacketUnconnectedPing {
    pub time: i64,
    pub magic: bool,
    pub guid: u64,
}

#[derive(Clone)]
pub struct PacketUnconnectedPong {
    pub time: i64,
    pub guid: u64,
    pub magic: bool,
    pub motd: String,
}

#[derive(Clone)]
pub struct OpenConnectionRequest1 {
    pub magic: bool,
    pub protocol_version: u8,
    pub mtu_size: u16,
}

#[derive(Clone)]
pub struct OpenConnectionRequest2 {
    pub magic: bool,
    pub address: std::net::SocketAddr,
    pub mtu: u16,
    pub guid: u64,
}

#[derive(Clone)]
pub struct OpenConnectionReply1 {
    pub magic: bool,
    pub guid: u64,
    pub use_encryption: u8,
    pub mtu_size: u16,
}

#[derive(Clone)]
pub struct OpenConnectionReply2 {
    pub magic: bool,
    pub guid: u64,
    pub address: std::net::SocketAddr,
    pub mtu: u16,
    pub encryption_enabled: u8,
}

#[derive(Clone)]
pub struct IncompatibleProtocolVersion {
    pub server_protocol: u8,
    pub magic: bool,
    pub server_guid: u64,
}

pub async fn read_packet_ping(buf : &[u8]) -> Result<PacketUnconnectedPing>{
    let mut cursor = Reader::new(buf);
    unwrap_or_return!(cursor.read_u8());
    Ok(PacketUnconnectedPing {
        time: unwrap_or_return!(cursor.read_i64(Endian::Big)),
        magic: unwrap_or_return!(cursor.read_magic()),
        guid: unwrap_or_return!(cursor.read_u64(Endian::Big)),
    })
}

pub async fn write_packet_ping(packet : &PacketUnconnectedPing) -> Result<Vec<u8>>{
    let mut cursor = Writer::new(vec![]);
    unwrap_or_return!(cursor.write_u8(transaction_packet_id_to_u8(PacketID::UnconnectedPing1)));
    unwrap_or_return!(cursor.write_i64(packet.time, Endian::Big));
    unwrap_or_return!(cursor.write_magic());
    unwrap_or_return!(cursor.write_u64(packet.guid, Endian::Big));
    Ok(cursor.get_raw_payload())
}

pub async fn read_packet_pong(buf : &[u8]) -> Result<PacketUnconnectedPong>{
    let mut cursor = Reader::new(buf);
    unwrap_or_return!(cursor.read_u8());
    Ok(PacketUnconnectedPong {
        time: unwrap_or_return!(cursor.read_i64(Endian::Big)),
        guid: unwrap_or_return!(cursor.read_u64(Endian::Big)),
        magic: unwrap_or_return!(cursor.read_magic()),
        motd: unwrap_or_return!(cursor.read_string()).to_owned(),
    })
}

pub async fn write_packet_pong(packet : &PacketUnconnectedPong) -> Result<Vec<u8>>{
    let mut cursor = Writer::new(vec![]);
    unwrap_or_return!(cursor.write_u8(transaction_packet_id_to_u8(PacketID::UnconnectedPong)));
    unwrap_or_return!(cursor.write_i64(packet.time, Endian::Big));
    unwrap_or_return!(cursor.write_u64(packet.guid, Endian::Big));
    unwrap_or_return!(cursor.write_magic());
    unwrap_or_return!(cursor.write_string(&packet.motd));
    Ok(cursor.get_raw_payload())
}

pub async fn read_packet_connection_open_request_1(buf : &[u8]) -> Result<OpenConnectionRequest1>{
    let mut cursor = Reader::new(buf);
    unwrap_or_return!(cursor.read_u8());
    Ok(OpenConnectionRequest1 {
        magic: unwrap_or_return!(cursor.read_magic()),
        protocol_version: unwrap_or_return!(cursor.read_u8()),
        mtu_size: (buf.len() + 29).try_into().unwrap(),
    })
}

pub async fn write_packet_connection_open_request_1(packet : &OpenConnectionRequest1) -> Result<Vec<u8>>{
    let mut cursor = Writer::new(vec![]);
    unwrap_or_return!(cursor.write_u8(transaction_packet_id_to_u8(PacketID::OpenConnectionRequest1)));
    unwrap_or_return!(cursor.write_magic());
    unwrap_or_return!(cursor.write_u8(packet.protocol_version));
    unwrap_or_return!(cursor
        .write(vec![0; (packet.mtu_size as usize) - (cursor.pos() as usize + 28) - 1].as_slice()));

    Ok(cursor.get_raw_payload())
}

pub async fn read_packet_connection_open_request_2(buf : &[u8]) -> Result<OpenConnectionRequest2>{
    let mut cursor = Reader::new(buf);
    unwrap_or_return!(cursor.read_u8());
    Ok(OpenConnectionRequest2 {
        magic: unwrap_or_return!(cursor.read_magic()),
        address: unwrap_or_return!(cursor.read_address()),
        mtu: unwrap_or_return!(cursor.read_u16(Endian::Big)),
        guid: unwrap_or_return!(cursor.read_u64(Endian::Big)),
    })
}

pub async fn write_packet_connection_open_request_2(packet : &OpenConnectionRequest2) -> Result<Vec<u8>>{
    let mut cursor = Writer::new(vec![]);
    unwrap_or_return!(cursor.write_u8(transaction_packet_id_to_u8(PacketID::OpenConnectionRequest2)));
    unwrap_or_return!(cursor.write_magic());
    unwrap_or_return!(cursor.write_address(packet.address));
    unwrap_or_return!(cursor.write_u16(packet.mtu, Endian::Big));
    unwrap_or_return!(cursor.write_u64(packet.guid, Endian::Big));

    Ok(cursor.get_raw_payload())
}

pub async fn read_packet_connection_open_reply_1(buf : &[u8]) -> Result<OpenConnectionReply1>{
    let mut cursor = Reader::new(buf);
    unwrap_or_return!(cursor.read_u8());
    Ok(OpenConnectionReply1 {
        magic: unwrap_or_return!(cursor.read_magic()),
        guid: unwrap_or_return!(cursor.read_u64(Endian::Big)),
        use_encryption: unwrap_or_return!(cursor.read_u8()),
        mtu_size: unwrap_or_return!(cursor.read_u16(Endian::Big)),
    })
}

pub async fn write_packet_connection_open_reply_1(packet : &OpenConnectionReply1) -> Result<Vec<u8>>{
    let mut cursor = Writer::new(vec![]);
    unwrap_or_return!(cursor.write_u8(transaction_packet_id_to_u8(PacketID::OpenConnectionReply1)));
    unwrap_or_return!(cursor.write_magic());
    unwrap_or_return!(cursor.write_u64(packet.guid, Endian::Big));
    unwrap_or_return!(cursor.write_u8(packet.use_encryption));
    unwrap_or_return!(cursor.write_u16(packet.mtu_size, Endian::Big));

    Ok(cursor.get_raw_payload())
}

pub async fn read_packet_connection_open_reply_2(buf : &[u8]) -> Result<OpenConnectionReply2>{
    let mut cursor = Reader::new(buf);
    unwrap_or_return!(cursor.read_u8());
    Ok(OpenConnectionReply2 {
        magic: unwrap_or_return!(cursor.read_magic()),
        guid: unwrap_or_return!(cursor.read_u64(Endian::Big)),
        address: unwrap_or_return!(cursor.read_address()),
        mtu: unwrap_or_return!(cursor.read_u16(Endian::Big)),
        encryption_enabled: unwrap_or_return!(cursor.read_u8()),
    })
}

pub async fn write_packet_connection_open_reply_2(packet : &OpenConnectionReply2) -> Result<Vec<u8>>{
    let mut cursor = Writer::new(vec![]);
    unwrap_or_return!(cursor.write_u8(transaction_packet_id_to_u8(PacketID::OpenConnectionReply2)));
    unwrap_or_return!(cursor.write_magic());
    unwrap_or_return!(cursor.write_u64(packet.guid, Endian::Big));
    unwrap_or_return!(cursor.write_address(packet.address));
    unwrap_or_return!(cursor.write_u16(packet.mtu, Endian::Big));
    unwrap_or_return!(cursor.write_u8(packet.encryption_enabled));


    Ok(cursor.get_raw_payload())
}

pub async fn read_packet_incompatible_protocol_version(buf : &[u8]) -> Result<IncompatibleProtocolVersion>{
    let mut cursor = Reader::new(buf);
    unwrap_or_return!(cursor.read_u8());
    Ok(IncompatibleProtocolVersion {
        server_protocol: unwrap_or_return!(cursor.read_u8()),
        magic: unwrap_or_return!(cursor.read_magic()),
        server_guid: unwrap_or_return!(cursor.read_u64(Endian::Big)),
    })
}

pub async fn write_packet_incompatible_protocol_version(packet : &IncompatibleProtocolVersion) -> Result<Vec<u8>>{
    let mut cursor = Writer::new(vec![]);
    unwrap_or_return!(cursor.write_u8(transaction_packet_id_to_u8(PacketID::IncompatibleProtocolVersion)));
    unwrap_or_return!(cursor.write_u8(packet.server_protocol));
    unwrap_or_return!(cursor.write_magic());
    unwrap_or_return!(cursor.write_u64(packet.server_guid, Endian::Big));

    Ok(cursor.get_raw_payload())
}