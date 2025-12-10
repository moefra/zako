use std::net::SocketAddr;

use crate::protobuf::net::IpAddress;

impl From<std::net::SocketAddr> for crate::protobuf::net::SocketAddress {
    fn from(ip: std::net::SocketAddr) -> Self {
        match ip {
            SocketAddr::V4(ip) => crate::protobuf::net::SocketAddress {
                ip: Some(IpAddress {
                    ip_addr: Some(crate::protobuf::net::ip_address::IpAddr::V4(u32::from(
                        *ip.ip(),
                    ))),
                }),
                port: ip.port() as u32,
            },
            SocketAddr::V6(ip) => crate::protobuf::net::SocketAddress {
                ip: Some(IpAddress {
                    ip_addr: Some(crate::protobuf::net::ip_address::IpAddr::V6(
                        ip.ip().octets().to_vec(),
                    )),
                }),
                port: ip.port() as u32,
            },
        }
    }
}
