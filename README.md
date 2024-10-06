# IPVS

High level crate to talk to IPVS over Generic Netlink.

Example (similar to `ipvsadm -ln`):

```rust
use ipvs::{DestinationExtended, IpvsClient, ServiceExtended};
fn main() {
    let c = IpvsClient::new().unwrap();
    for service in c.get_all_services().unwrap() {
        let dests = c.get_all_destinations(&service).unwrap();
        print_ipvs_service(&service, dests.as_slice());
    }
}
```
