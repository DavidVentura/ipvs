use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    time::Duration,
};

use ipvs::{self, AddressFamily, Destination, Flags, ForwardTypeFull, Netmask};

struct TestEnv {
    accepted_addr: SocketAddr,
    refused_addr: SocketAddr,
    dropped_addr: SocketAddr,

    _accept_server: TcpListener,
    //    _mutex: MutexGuard<'a, i32>,
}

impl TestEnv {
    pub fn new() -> TestEnv {
        let c = ipvs::IpvsClient::new().unwrap();
        let localhostv4 = Ipv4Addr::new(127, 0, 0, 1);
        let localhost = std::net::IpAddr::V4(localhostv4);
        let accepted = ipvs::Service {
            address: localhost,
            netmask: Netmask::new(32, AddressFamily::IPv4),
            scheduler: ipvs::Scheduler::RoundRobin,
            flags: Flags(0),
            port: Some(33),
            fw_mark: None,
            persistence_timeout: None,
            family: AddressFamily::IPv4,
            protocol: ipvs::Protocol::TCP,
        };
        let refused = ipvs::Service {
            port: Some(44),
            ..accepted
        };
        let dropped = ipvs::Service {
            port: Some(55),
            ..accepted
        };

        let _ = c.delete_service(&accepted);
        let _ = c.delete_service(&refused);
        let _ = c.delete_service(&dropped);

        c.create_service(&accepted).unwrap();
        c.create_service(&refused).unwrap();
        c.create_service(&dropped).unwrap();

        let accept_port = 1234;
        let accept_dest = Destination {
            address: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            fwd_method: ForwardTypeFull::Masquerade,
            weight: 1,
            upper_threshold: None,
            lower_threshold: None,
            port: accept_port,
            family: AddressFamily::IPv4,
        };
        let refused_dest = ipvs::Destination {
            port: 2345,
            ..accept_dest
        };
        // unroutable address, TEST-NET-3
        let dropped_dest = ipvs::Destination {
            address: std::net::IpAddr::V4(Ipv4Addr::new(203, 0, 113, 2)),
            ..accept_dest
        };

        let _ = c.delete_destination(&accepted, &accept_dest);
        let _ = c.delete_destination(&refused, &refused_dest);
        let _ = c.delete_destination(&dropped, &dropped_dest);

        c.create_destination(&accepted, &accept_dest).unwrap();
        c.create_destination(&refused, &refused_dest).unwrap();
        c.create_destination(&dropped, &dropped_dest).unwrap();

        let _accept_server = TcpListener::bind(format!("127.0.0.1:{accept_port}")).unwrap();
        TestEnv {
            accepted_addr: SocketAddr::V4(SocketAddrV4::new(localhostv4, 33)),
            refused_addr: SocketAddr::V4(SocketAddrV4::new(localhostv4, 44)),
            dropped_addr: SocketAddr::V4(SocketAddrV4::new(localhostv4, 55)),
            _accept_server,
            //_mutex: test_mutex.lock().unwrap(),
        }
    }
}

#[test]
fn test_successful_dest() {
    let te = TestEnv::new();
    let _client = TcpStream::connect(te.accepted_addr).unwrap();
}

#[test]
fn test_rejected_dest() {
    let te = TestEnv::new();
    match TcpStream::connect(te.refused_addr) {
        Ok(_) => panic!("Expected error"),
        Err(e) => match e.kind() {
            std::io::ErrorKind::ConnectionRefused => (),
            _ => panic!("Expected ConnectionRefused"),
        },
    }
}

#[test]
fn test_dropped_dest() {
    let te = TestEnv::new();
    match TcpStream::connect_timeout(&te.dropped_addr, Duration::from_millis(100)) {
        Ok(_) => panic!("Expected error"),
        Err(e) => match e.kind() {
            std::io::ErrorKind::TimedOut => (),
            other => panic!("Expected TimedOut, got {other}"),
        },
    }
}
