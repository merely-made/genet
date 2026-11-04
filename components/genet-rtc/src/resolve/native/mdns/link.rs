use super::*;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};

pub(super) fn index(interface: Ipv4Addr) -> Option<u32> {
    NetworkInterface::show()
        .ok()?
        .into_iter()
        .find(|i| i.addr.iter().any(|a| a.ip() == IpAddr::V4(interface)))
        .map(|i| i.index)
}
pub(super) fn prefix(interface: Ipv4Addr) -> Result<u8, ResolveError> {
    if interface.is_loopback() {
        return Ok(8);
    }
    NetworkInterface::show()
        .map_err(|_| ResolveError::Unavailable)?
        .into_iter()
        .flat_map(|i| i.addr)
        .find_map(|a| {
            let Addr::V4(a) = a else {
                return None;
            };
            if a.ip != interface {
                return None;
            }
            let mask = u32::from(a.netmask?);
            let len = mask.leading_ones();
            (len > 0 && mask == u32::MAX << (32 - len)).then_some(len as u8)
        })
        .ok_or(ResolveError::Unavailable)
}
