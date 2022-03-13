use std::{io::Result , net::{SocketAddr}, sync::Arc};
use tokio::net::UdpSocket;
use rand;

use crate::{packet::*, utils::*};

pub struct RaknetSocket{
    local_addr : SocketAddr,
    peer_addr : SocketAddr,
    s : Arc<UdpSocket>,
    guid : u64
}

impl RaknetSocket {
    pub fn from(addr : &SocketAddr , s : &Arc<UdpSocket>) -> Self {
        Self{
            peer_addr : addr.clone(),
            local_addr : s.local_addr().unwrap(),
            s: s.clone(),
            guid : rand::random()
        }
    }

    pub async fn connect(addr : &SocketAddr) -> Result<Self>{

        let guid : u64 = rand::random();

        let s = match UdpSocket::bind("0.0.0.0:0").await{
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let packet = OpenConnectionRequest1{
            magic: true,
            protocol_version: RAKNET_PROTOCOL_VERSION,
            mtu_size: 1500,
        };

        let buf = write_packet_connection_open_request_1(&packet).await.unwrap();

        s.send_to(&buf, addr).await.unwrap();

        let mut buf = [0u8 ; 2048];
        let (size ,src ) = s.recv_from(&mut buf).await.unwrap();

        if buf[0] != transaction_packet_id_to_u8(PacketID::OpenConnectionReply1){
            if buf[0] == transaction_packet_id_to_u8(PacketID::IncompatibleProtocolVersion){
                let packet = match read_packet_incompatible_protocol_version(&buf[..size]).await{
                    Ok(p) => p,
                    Err(e) => return Err(e),
                };

                return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("server version : {}", packet.server_protocol)));
            }else{
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "open connection reply2 packetid incorrect"));
            }
        }

        let reply1 = match read_packet_connection_open_reply_1(&buf[..size]).await{
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let packet = OpenConnectionRequest2{
            magic: true,
            address: src,
            mtu: reply1.mtu_size,
            guid: guid,
        };

        let buf = write_packet_connection_open_request_2(&packet).await.unwrap();

        s.send_to(&buf, addr).await.unwrap();

        let mut buf = [0u8 ; 2048];
        let (size ,_ ) = s.recv_from(&mut buf).await.unwrap();

        if buf[0] != transaction_packet_id_to_u8(PacketID::OpenConnectionReply2){
            if buf[0] == transaction_packet_id_to_u8(PacketID::IncompatibleProtocolVersion){

                let packet = match read_packet_incompatible_protocol_version(&buf[..size]).await{
                    Ok(p) => p,
                    Err(e) => return Err(e),
                };

                return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("server only support protocol version : {}", packet.server_protocol)));
            }else{
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "open connection reply2 packetid incorrect"));
            }
        }

        let _reply2 = match read_packet_connection_open_reply_2(&buf[..size]).await{
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        Ok(RaknetSocket{
            peer_addr : addr.clone(),
            local_addr : s.local_addr().unwrap(),
            s: Arc::new(s),
            guid : guid
        })
    }

    pub async fn ping(addr : &SocketAddr) -> Result<i64> {
        let packet = PacketUnconnectedPing{
            time: cur_timestamp(),
            magic: true,
            guid: rand::random(),
        };

        let s = match UdpSocket::bind("0.0.0.0:0").await{
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let buf = write_packet_ping(&packet).await.unwrap();

        s.send_to(buf.as_slice(), addr).await.unwrap();

        let mut buf = [0u8 ; 1024];
        match s.recv_from(&mut buf).await{
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let pong = match read_packet_pong(&buf).await{
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        Ok(pong.time - packet.time)
    }

    pub fn handle_packet(&self , _buf : &[u8]){

    }

    pub fn peer_addr(&self) -> Result<SocketAddr>{
        Ok(self.peer_addr)
    }

    pub fn local_addr(&self) -> Result<SocketAddr>{
        Ok(self.local_addr)
    }
}