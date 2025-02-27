pub mod rpc_definitions {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use ipnetwork::IpNetwork;
    use thiserror::Error;

    pub mod shared {
        tonic::include_proto!("attacks.shared");
    }

    tonic::include_proto!("attacks");

    impl From<shared::Ipv4> for Ipv4Addr {
        fn from(value: shared::Ipv4) -> Self {
            Ipv4Addr::from(value.address.to_le_bytes())
        }
    }

    impl From<Ipv4Addr> for shared::Ipv4 {
        fn from(value: Ipv4Addr) -> Self {
            shared::Ipv4 {
                address: i32::from_le_bytes(value.octets()),
            }
        }
    }

    impl From<shared::Ipv6> for Ipv6Addr {
        fn from(value: shared::Ipv6) -> Self {
            let [a, b, c, d, e, f, g, h] = value.part0.to_le_bytes();
            let [i, j, k, l, m, n, o, p] = value.part1.to_le_bytes();
            Ipv6Addr::from([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p])
        }
    }

    impl From<Ipv6Addr> for shared::Ipv6 {
        fn from(value: Ipv6Addr) -> Self {
            let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = value.octets();
            shared::Ipv6 {
                part0: i64::from_le_bytes([a, b, c, d, e, f, g, h]),
                part1: i64::from_le_bytes([i, j, k, l, m, n, o, p]),
            }
        }
    }

    impl From<IpAddr> for shared::Address {
        fn from(value: IpAddr) -> Self {
            Self {
                address: Some(match value {
                    IpAddr::V4(addr) => shared::address::Address::Ipv4(addr.into()),
                    IpAddr::V6(addr) => shared::address::Address::Ipv6(addr.into()),
                }),
            }
        }
    }

    #[derive(Debug, Error)]
    #[error("Address was None")]
    pub struct AddressConvError;

    impl TryFrom<shared::Address> for IpAddr {
        type Error = AddressConvError;

        fn try_from(value: shared::Address) -> Result<Self, Self::Error> {
            let shared::Address { address } = value;
            Ok(match address.ok_or(AddressConvError)? {
                shared::address::Address::Ipv4(v) => IpAddr::from(Ipv4Addr::from(v)),
                shared::address::Address::Ipv6(v) => IpAddr::from(Ipv6Addr::from(v)),
            })
        }
    }

    impl From<IpNetwork> for shared::Net {
        fn from(value: IpNetwork) -> Self {
            Self {
                net: match value {
                    IpNetwork::V4(x) => Some(shared::net::Net::Ipv4net(shared::Ipv4Net {
                        address: Some(x.ip().into()),
                        netmask: i32::from_le_bytes(x.mask().octets()),
                    })),
                    IpNetwork::V6(x) => {
                        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = x.mask().octets();
                        Some(shared::net::Net::Ipv6net(shared::Ipv6Net {
                            address: Some(x.ip().into()),
                            netmask0: i64::from_le_bytes([a, b, c, d, e, f, g, h]),
                            netmask1: i64::from_le_bytes([i, j, k, l, m, n, o, p]),
                        }))
                    }
                },
            }
        }
    }

    impl From<IpNetwork> for shared::NetOrAddress {
        fn from(value: IpNetwork) -> Self {
            Self {
                net_or_address: Some(shared::net_or_address::NetOrAddress::Net(value.into())),
            }
        }
    }
}
