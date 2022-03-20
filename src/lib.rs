mod socket;
mod packet;
mod utils;
mod datatype;
mod arq;
mod fragment;
pub mod server;

pub use crate::server::*;
pub use crate::socket::*;

#[tokio::test]
async fn test_ping_pong(){

    let s = tokio::net::UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let port = s.local_addr().unwrap().port();

    tokio::spawn(async move {
        let mut buf = [0u8 ; 1024];
        let (size , addr ) = s.recv_from(&mut buf).await.unwrap();

        let _pong = packet::read_packet_ping(&buf[..size]).await.unwrap();
        let packet = packet::PacketUnconnectedPong{
            time: utils::cur_timestamp(),
            magic: true,
            guid: rand::random(),
            motd : format!("MCPE;Dedicated Server;486;1.18.11;0;10;12322747879247233720;Bedrock level;Survival;1;{};", s.local_addr().unwrap().port())
        };

        let buf = packet::write_packet_pong(&packet).await.unwrap();

        s.send_to(buf.as_slice(), addr).await.unwrap();
    });

    let addr = format!("127.0.0.1:{}", port);
    let latency = socket::RaknetSocket::ping(&addr.as_str().parse().unwrap()).await.unwrap();
    assert!(latency < 10 && latency >= 0);
}

#[tokio::test]
async fn test_connect(){
    let mut server = RaknetListener::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let local_addr = server.local_addr().unwrap();
    server.listen().await;
    tokio::spawn(async move {
        let mut client1 = server.accept().await.unwrap();
        assert!(client1.local_addr().unwrap() == local_addr);
        client1.send(&[1,2,3]).await.unwrap();
    });
    let mut client2 = RaknetSocket::connect(&local_addr).await.unwrap();
    assert!(client2.peer_addr().unwrap() == local_addr);
    let buf = client2.recv().await.unwrap();
    assert!(buf == vec![1,2,3]);
}

/* 
#[tokio::test]
async fn chore(){
    let mut server = RaknetListener::bind("0.0.0.0:19132".parse().unwrap()).await.unwrap();
    server.listen().await;
    loop{
        let mut client1 = server.accept().await.unwrap();
        let ret = client1.recv().await.unwrap();
        dbg!(ret);
    }
}
*/