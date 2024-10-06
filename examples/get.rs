use std::net::Ipv4Addr;

use ipvs::IpvsClient;
use ipvs::*;
fn main() {
    let c = IpvsClient::new().unwrap();
    //let service = Service {
    //    address: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    //    netmask: Netmask::new(32, AddressFamily::IPv4),
    //    scheduler: Scheduler::RoundRobin,
    //    flags: Flags(2),
    //    port: Some(22),
    //    fw_mark: None,
    //    persistence_timeout: None,
    //    family: AddressFamily::IPv4,
    //    protocol: Protocol::TCP,
    //};
    for service in c.get_all_services().unwrap() {
        println!("Service {service:?}");
        let dests = c.get_all_destinations(&service).unwrap();
        for dest in dests {
            println!("  Destination {dest:?}");
        }
    }
}
