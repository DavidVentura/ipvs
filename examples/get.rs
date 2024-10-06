use ipvs::IpvsClient;
fn main() {
    let c = IpvsClient::new().unwrap();
    for service in c.get_all_services().unwrap() {
        println!("Service {:#?}", service.service);
        let dests = c.get_all_destinations(&service).unwrap();
        for dest in dests {
            println!("  Destination {:#?}", dest.destination);
        }
    }
}
