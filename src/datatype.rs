use bytes::{BufMut , Buf};
use std::{
    io::{Cursor, Result, Read},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str,
};

use crate::utils::Endian;

#[derive(Clone)]
pub struct RaknetWriter {
    buf: Vec<u8>,
}

impl RaknetWriter {

    pub fn new() -> Self {
        Self {
            buf : vec![],
        }
    }

    pub async fn write(&mut self, v: &[u8]) -> Result<()> {
        Ok(self.buf.put_slice(v))
    }

    pub async fn write_u8(&mut self, v: u8) -> Result<()> {
        Ok(self.buf.put_u8(v))
    }

    pub async fn write_i16(&mut self, v: i16, n: Endian) -> Result<()> {
        match n {
            Endian::Big => Ok(self.buf.put_i16(v)),
            Endian::Little => Ok(self.buf.put_i16_le(v))
        }
    }

    pub async fn write_u16(&mut self, v: u16, n: Endian) -> Result<()> {
        match n {
            Endian::Big => Ok(self.buf.put_u16(v)),
            Endian::Little => Ok(self.buf.put_u16_le(v))
        }
    }

    pub async fn write_u24(&mut self, v: u32, n: Endian) -> Result<()> {
        match n {
            Endian::Big => {
                let a = v.to_be_bytes();
                self.buf.put_u8(a[1]);
                self.buf.put_u8(a[2]);
                self.buf.put_u8(a[3]);
            },
            Endian::Little => {
                let a = v.to_le_bytes();
                self.buf.put_u8(a[0]);
                self.buf.put_u8(a[1]);
                self.buf.put_u8(a[2]);
            },
        }
        Ok(())
    }

    pub async fn write_u32(&mut self, v: u32, n: Endian) -> Result<()> {
        match n {
            Endian::Big => Ok(self.buf.put_u32(v)),
            Endian::Little => Ok(self.buf.put_u32_le(v))
        }
    }

    pub async fn write_i32(&mut self, v: i32, n: Endian) -> Result<()> {
        match n {
            Endian::Big => Ok(self.buf.put_i32(v)),
            Endian::Little => Ok(self.buf.put_i32_le(v))
        }
    }

    pub async fn write_i64(&mut self, v: i64, n: Endian) -> Result<()> {
        match n {
            Endian::Big => Ok(self.buf.put_i64(v)),
            Endian::Little => Ok(self.buf.put_i64_le(v))
        }
    }

    pub async fn write_magic(&mut self) -> Result<usize> {
        let magic: [u8;16] = [
            0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34,
            0x56, 0x78,
        ];
        self.buf.put_slice(&magic);
        Ok(magic.len())
    }

    pub async fn write_u64(&mut self, v: u64, n: Endian) -> Result<()> {
        match n {
            Endian::Big => Ok(self.buf.put_u64(v)),
            Endian::Little => Ok(self.buf.put_u64_le(v)),
        }
    }

    pub async fn write_string(&mut self, body: &str) -> Result<()> {
        let raw = body.as_bytes();
        self.buf.put_u16(raw.len() as u16);
        Ok(self.buf.put_slice(raw))
    }

    pub async fn write_address(&mut self, address: SocketAddr) -> Result<()> {
        if address.is_ipv4() {
            self.write_u8(0x4).await?;
            let ip_bytes = match address.ip() {
                IpAddr::V4(ip) => ip.octets().to_vec(),
                _ => vec![0; 4],
            };

            self.write_u8(0xff - ip_bytes[0]).await?;
            self.write_u8(0xff - ip_bytes[1]).await?;
            self.write_u8(0xff - ip_bytes[2]).await?;
            self.write_u8(0xff - ip_bytes[3]).await?;
            self.write_u16(address.port() , Endian::Big).await?;
            Ok(())
        } else {
            self.write_i16(23 , Endian::Little).await?;
            self.write_u16(address.port() , Endian::Big).await?;
            self.write_i32(0, Endian::Big).await?;
            let ip_bytes = match address.ip() {
                IpAddr::V6(ip) => ip.octets().to_vec(),
                _ => vec![0; 16],
            };
            self.write(&ip_bytes).await?;
            self.write_i32(0 , Endian::Big).await?;
            Ok(())
        }
    }

    pub fn get_raw_payload(self) -> Vec<u8> {
        self.buf
    }

    pub fn pos(&self) -> u64 {
        self.buf.len() as u64
    }
}

pub struct RaknetReader {
    buf: Cursor<Vec<u8>>,
}

impl RaknetReader {
    pub fn new(buf : Vec<u8>) -> Self {
        Self {
            buf: Cursor::new(buf)
        }
    }
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        self.buf.read_exact(buf)
    }
    pub async fn read_u8(&mut self) -> Result<u8> {
        Ok(self.buf.get_u8())
    }

    pub async fn read_u16(&mut self, n: Endian) -> Result<u16> {
        match n {
            Endian::Big => Ok(self.buf.get_u16()),
            Endian::Little => Ok(self.buf.get_u16_le()),
        }
    }

    pub async fn read_u24(&mut self , n : Endian) -> Result<u32>{
        match n {
            Endian::Big => {

                let a = self.buf.get_u8();
                let b = self.buf.get_u8();
                let c = self.buf.get_u8();

                let ret = u32::from_be_bytes([0 , a, b ,c]);
                Ok(ret)
            },
            Endian::Little => {
                let a = self.buf.get_u8();
                let b = self.buf.get_u8();
                let c = self.buf.get_u8();

                let ret = u32::from_le_bytes([a , b , c , 0]);
                Ok(ret)
            },
        }
    }

    pub async fn read_u32(&mut self, n: Endian) -> Result<u32> {
        match n {
            Endian::Big => Ok(self.buf.get_u32()),
            Endian::Little => Ok(self.buf.get_u32_le()),
        }
    }

    pub async fn read_u64(&mut self, n: Endian) -> Result<u64> {
        match n {
            Endian::Big => Ok(self.buf.get_u64()),
            Endian::Little => Ok(self.buf.get_u64_le()),
        }
    }
    pub async fn read_i64(&mut self, n: Endian) -> Result<i64> {
        match n {
            Endian::Big => Ok(self.buf.get_i64()),
            Endian::Little => Ok(self.buf.get_i64_le()),
        }
    }

    pub async fn read_string(&mut self) -> Result<String> {
        let size = self.read_u16(Endian::Big).await?;
        let mut buf = vec![0u8 ; size as usize].into_boxed_slice();
        self.buf.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf.to_vec()).unwrap())
    }

    pub async fn read_magic(&mut self) -> Result<bool> {
        let mut magic = [0; 16];
        self.buf.read_exact(&mut magic)?;
        let offline_magic = [
            0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34,
            0x56, 0x78,
        ];
        Ok(magic == offline_magic)
    }
    
    pub async fn read_address(&mut self) -> Result<SocketAddr> {
        let ip_ver = self.read_u8().await?;

        if ip_ver == 4 {
            let ip = Ipv4Addr::new(
                0xff - self.read_u8().await?,
                0xff - self.read_u8().await?,
                0xff - self.read_u8().await?,
                0xff - self.read_u8().await?,
            );
            let port = self.read_u16(Endian::Big).await?;
            Ok(SocketAddr::new(IpAddr::V4(ip), port))
        } else {
            self.next(2);
            let port = self.read_u16(Endian::Big).await?;
            self.next(4);
            let mut addr_buf = [0; 16];
            self.buf.read_exact(&mut addr_buf)?;

            let mut address_cursor = RaknetReader::new(addr_buf.to_vec());
            self.next(4);
            Ok(SocketAddr::new(
                IpAddr::V6(Ipv6Addr::new(
                    address_cursor.read_u16(Endian::Big).await?,
                    address_cursor.read_u16(Endian::Big).await?,
                    address_cursor.read_u16(Endian::Big).await?,
                    address_cursor.read_u16(Endian::Big).await?,
                    address_cursor.read_u16(Endian::Big).await?,
                    address_cursor.read_u16(Endian::Big).await?,
                    address_cursor.read_u16(Endian::Big).await?,
                    address_cursor.read_u16(Endian::Big).await?,
                )),
                port,
            ))
        } //IPv6 address = 128bit = u8 * 16
    }

    pub fn next(&mut self, n: u64) {
        self.buf.set_position(self.buf.position() + n);
    }

    pub fn pos(&self) -> u64 {
        self.buf.position()
    }
}


#[tokio::test]
async fn test_u24_encode_decode(){

    let a : u32 = 65535*21; 
    let b = a.to_le_bytes();
    let mut reader = RaknetReader::new(b.to_vec());

    let c = reader.read_u24(Endian::Little).await.unwrap();

    assert!(a == c);

    let mut writer = RaknetWriter::new();
    writer.write_u24(a, Endian::Little).await.unwrap();

    let buf = writer.get_raw_payload();
    let mut reader = RaknetReader::new(buf);

    let c = reader.read_u24(Endian::Little).await.unwrap();

    assert!(a == c);
}