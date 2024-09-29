// SPDX-License-Identifier: MIT

use std::error::Error;

use netlink_packet_core::{NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST};
use netlink_packet_generic::ctrl::nlas::GenlCtrlAttrs;
use netlink_packet_generic::ctrl::{GenlCtrl, GenlCtrlCmd};
use netlink_packet_generic::{GenlFamily, GenlMessage};
pub use netlink_packet_ipvs::ctrl::nlas::destination::{Destination, ForwardTypeFull};
pub use netlink_packet_ipvs::ctrl::nlas::service::{Flags, Netmask, Protocol, Scheduler, Service};
pub use netlink_packet_ipvs::ctrl::nlas::AddressFamily;
use netlink_packet_ipvs::ctrl::nlas::IpvsCtrlAttrs;
use netlink_packet_ipvs::ctrl::{IpvsCtrlCmd, IpvsServiceCtrl};
use netlink_sys::{protocols::NETLINK_GENERIC, Socket, SocketAddr};

pub struct IpvsClient {
    socket: Socket,
    family_id: u16,
}
impl IpvsClient {
    pub fn new() -> Result<IpvsClient, Box<dyn Error>> {
        let mut socket = Socket::new(NETLINK_GENERIC)?;
        socket.bind_auto()?;
        socket.connect(&SocketAddr::new(0, 0))?;
        let mut genlmsg = GenlMessage::from_payload(GenlCtrl {
            cmd: GenlCtrlCmd::GetFamily,
            nlas: vec![GenlCtrlAttrs::FamilyName("IPVS\0".to_string())],
        });
        genlmsg.finalize();
        let mut nlmsg = NetlinkMessage::from(genlmsg);
        nlmsg.header.flags = NLM_F_REQUEST | NLM_F_DUMP;
        nlmsg.finalize();
        let mut txbuf = vec![0u8; nlmsg.buffer_len()];
        nlmsg.serialize(&mut txbuf);
        let r = send_fam_buf(&socket, &txbuf)?;
        let mut family_id = 0;
        let mut good = false;
        let mut found = false;
        for entry in r {
            match entry {
                GenlCtrlAttrs::FamilyName(f) => {
                    good = f == IpvsServiceCtrl::family_name();
                }
                GenlCtrlAttrs::FamilyId(i) => {
                    if good {
                        found = true;
                        family_id = i;
                        break;
                    }
                }
                _ => (),
            }
        }
        if !found {
            return Err(
                "IPVS not found -- is IP_VS enabled in kernel config? module loaded?".into(),
            );
        }

        Ok(IpvsClient { socket, family_id })
    }
    pub fn create_service(&self, svc: &Service) -> std::io::Result<()> {
        let txbuf = IpvsServiceCtrl {
            cmd: IpvsCtrlCmd::NewService,
            nlas: vec![IpvsCtrlAttrs::Service(svc.create_nlas())],
            family_id: self.family_id,
        }
        .serialize(false);
        send_buf(&self.socket, &txbuf)?;
        Ok(())
    }
    pub fn delete_service(&self, svc: &Service) -> std::io::Result<()> {
        let txbuf = IpvsServiceCtrl {
            cmd: IpvsCtrlCmd::DelService,
            nlas: vec![IpvsCtrlAttrs::Service(svc.create_nlas())],
            family_id: self.family_id,
        }
        .serialize(false);
        send_buf(&self.socket, &txbuf)?;
        Ok(())
    }
    pub fn update_service(&self, svc: &Service, to: &Service) -> std::io::Result<Service> {
        let txbuf = IpvsServiceCtrl {
            cmd: IpvsCtrlCmd::SetService,
            nlas: vec![
                IpvsCtrlAttrs::Service(svc.create_nlas()),
                IpvsCtrlAttrs::Service(to.create_nlas()),
            ],
            family_id: self.family_id,
        }
        .serialize(false);
        let mut r = send_buf(&self.socket, &txbuf)?;
        assert!(r.len() == 1);
        let entry = r.pop().unwrap();
        match entry {
            IpvsCtrlAttrs::Service(nlas) => {
                let s = Service::from_nlas(&nlas).unwrap();
                return Ok(s);
            }
            IpvsCtrlAttrs::Destination(_) => {
                panic!("unreachable");
            }
        }
    }
    pub fn get_all_services(&self) -> std::io::Result<Vec<Service>> {
        let txbuf = IpvsServiceCtrl {
            cmd: IpvsCtrlCmd::GetService,
            nlas: vec![],
            family_id: self.family_id,
        }
        .serialize(true);
        let r = send_buf(&self.socket, &txbuf)?;
        let mut ret = vec![];
        for entry in r {
            match entry {
                IpvsCtrlAttrs::Service(nlas) => {
                    // FIXME unwrap
                    let s = Service::from_nlas(&nlas).unwrap();
                    ret.push(s);
                }
                IpvsCtrlAttrs::Destination(_) => {
                    panic!("unreachable");
                }
            }
        }
        Ok(ret)
    }
    pub fn create_destination(&self, svc: &Service, dst: &Destination) -> std::io::Result<()> {
        let txbuf = IpvsServiceCtrl {
            cmd: IpvsCtrlCmd::NewDest,
            nlas: vec![
                IpvsCtrlAttrs::Service(svc.create_nlas()),
                IpvsCtrlAttrs::Destination(dst.create_nlas()),
            ],
            family_id: self.family_id,
        }
        .serialize(false);
        send_buf(&self.socket, &txbuf)?;
        Ok(())
    }
    pub fn delete_destination(&self, svc: &Service, dst: &Destination) -> std::io::Result<()> {
        let txbuf = IpvsServiceCtrl {
            cmd: IpvsCtrlCmd::DelDest,
            nlas: vec![
                IpvsCtrlAttrs::Service(svc.create_nlas()),
                IpvsCtrlAttrs::Destination(dst.create_nlas()),
            ],
            family_id: self.family_id,
        }
        .serialize(false);
        send_buf(&self.socket, &txbuf)?;
        Ok(())
    }
    pub fn update_destination(
        &self,
        svc: &Service,
        dst: &Destination,
        to: &Destination,
    ) -> std::io::Result<Service> {
        let txbuf = IpvsServiceCtrl {
            cmd: IpvsCtrlCmd::SetDest,
            nlas: vec![
                IpvsCtrlAttrs::Service(svc.create_nlas()),
                IpvsCtrlAttrs::Destination(dst.create_nlas()),
                IpvsCtrlAttrs::Destination(to.create_nlas()),
            ],
            family_id: self.family_id,
        }
        .serialize(false);
        let mut r = send_buf(&self.socket, &txbuf)?;
        assert!(r.len() == 1);
        let entry = r.pop().unwrap();
        match entry {
            IpvsCtrlAttrs::Service(nlas) => {
                let s = Service::from_nlas(&nlas).unwrap();
                return Ok(s);
            }
            IpvsCtrlAttrs::Destination(_) => {
                panic!("unreachable");
            }
        }
    }
    pub fn get_all_destinations(&self, svc: &Service) -> std::io::Result<Vec<Destination>> {
        let txbuf = IpvsServiceCtrl {
            cmd: IpvsCtrlCmd::GetDest,
            nlas: vec![IpvsCtrlAttrs::Service(svc.create_nlas())],
            family_id: self.family_id,
        }
        .serialize(true);
        let r = send_buf(&self.socket, &txbuf)?;
        let mut ret = vec![];
        for entry in r {
            match entry {
                IpvsCtrlAttrs::Service(_) => {
                    panic!("unreachable");
                }
                IpvsCtrlAttrs::Destination(nlas) => {
                    // FIXME unwrap
                    let s = Destination::from_nlas(&nlas).unwrap();
                    ret.push(s);
                }
            }
        }
        Ok(ret)
    }
}

// TODO: send_buf and send_fam_buf should be generic
// but `payloads.nlas` can't be accessed generically
fn send_buf(socket: &Socket, buf: &[u8]) -> Result<Vec<IpvsCtrlAttrs>, std::io::Error> {
    socket.send(&buf, 0).unwrap();

    let mut ret = Vec::new();
    loop {
        let mut offset = 0;
        let (rxbuf, _) = socket.recv_from_full().unwrap();

        loop {
            let buf = &rxbuf[offset..];
            let msg = <NetlinkMessage<GenlMessage<IpvsServiceCtrl>>>::deserialize(buf).unwrap();

            match msg.payload {
                NetlinkPayload::Done(_) => {
                    return Ok(ret);
                }
                NetlinkPayload::InnerMessage(genlmsg) => {
                    ret.extend_from_slice(&genlmsg.payload.nlas);
                }
                NetlinkPayload::Error(err) => {
                    if err.code.is_some() {
                        let e = std::io::Error::from_raw_os_error(err.code.unwrap().get().abs());
                        return Err(e);
                    }
                }
                other => {
                    println!("{:?}", other)
                }
            }

            offset += msg.header.length as usize;
            if offset >= rxbuf.len() || msg.header.length == 0 {
                break;
            }
        }
    }
}

fn send_fam_buf(socket: &Socket, buf: &[u8]) -> Result<Vec<GenlCtrlAttrs>, std::io::Error> {
    socket.send(&buf, 0).unwrap();

    let mut ret = Vec::new();
    loop {
        let mut offset = 0;
        let (rxbuf, _) = socket.recv_from_full().unwrap();

        loop {
            let buf = &rxbuf[offset..];
            let msg = <NetlinkMessage<GenlMessage<GenlCtrl>>>::deserialize(buf).unwrap();

            match msg.payload {
                NetlinkPayload::Done(_) => {
                    return Ok(ret);
                }
                NetlinkPayload::InnerMessage(genlmsg) => {
                    ret.extend_from_slice(&genlmsg.payload.nlas);
                }
                NetlinkPayload::Error(err) => {
                    if err.code.is_some() {
                        let e = std::io::Error::from_raw_os_error(err.code.unwrap().get().abs());
                        return Err(e);
                    }
                }
                other => {
                    println!("{:?}", other)
                }
            }

            offset += msg.header.length as usize;
            if offset >= rxbuf.len() || msg.header.length == 0 {
                break;
            }
        }
    }
}
