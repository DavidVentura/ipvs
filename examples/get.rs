use ipvs::{DestinationExtended, IpvsClient, ServiceExtended};
fn main() {
    let c = IpvsClient::new().unwrap();
    for service in c.get_all_services().unwrap() {
        let dests = c.get_all_destinations(&service).unwrap();
        print_ipvs_service(&service, dests.as_slice());
    }
}
pub fn print_ipvs_service(service: &ServiceExtended, destinations: &[DestinationExtended]) {
    println!("IP Virtual Server version 1.2.1 (size=4096)");
    println!("Prot LocalAddress:Port\t\tConns   InPkts  OutPkts  InBytes OutBytes");
    println!("  -> RemoteAddress:Port");

    let stats = &service.stats64;
    println!(
        "{:<4} {:21} {:8} {:8} {:8} {:8} {:8}",
        format!("{:?}", service.service.protocol),
        format!(
            "{}:{}",
            service.service.address,
            service.service.port.unwrap_or(0)
        ),
        stats.connections,
        stats.incoming_packets,
        stats.outgoing_packets,
        stats.incoming_bytes,
        stats.outgoing_bytes
    );

    for dest in destinations {
        let stats = &dest.stats64;
        println!(
            "  -> {:21} {:8} {:8} {:8} {:8} {:8}",
            format!("{}:{}", dest.destination.address, dest.destination.port),
            stats.connections,
            stats.incoming_packets,
            stats.outgoing_packets,
            stats.incoming_bytes,
            stats.outgoing_bytes
        );
    }
}
