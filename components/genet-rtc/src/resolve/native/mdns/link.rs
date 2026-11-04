use super::*;

pub(super) fn prefix(interface: Ipv4Addr) -> Result<u8, ResolveError> {
    if interface.is_loopback() {
        return Ok(8);
    }
    system_prefix(interface).ok_or(ResolveError::Unavailable)
}
#[cfg(windows)]
fn system_prefix(interface: Ipv4Addr) -> Option<u8> {
    ipconfig::get_adapters()
        .ok()?
        .into_iter()
        .filter(|a| {
            a.oper_status() == ipconfig::OperStatus::IfOperStatusUp
                && a.ip_addresses().contains(&IpAddr::V4(interface))
        })
        .flat_map(|a| a.prefixes().to_vec())
        .filter_map(|(ip, len)| {
            let IpAddr::V4(ip) = ip else {
                return None;
            };
            if !(1..=32).contains(&len) {
                return None;
            }
            let mask = u32::MAX << (32 - len);
            (u32::from(ip) & mask == u32::from(interface) & mask).then_some(len as u8)
        })
        .min()
}
#[cfg(unix)]
fn system_prefix(interface: Ipv4Addr) -> Option<u8> {
    let mut list = std::ptr::null_mut();
    // SAFETY: getifaddrs initializes an owned list, inspected while alive and
    // freed exactly once below. Read only nonnull IPv4 address/netmask pairs.
    unsafe {
        if libc::getifaddrs(&mut list) != 0 {
            return None;
        }
        let mut cursor = list;
        let mut result = None;
        while !cursor.is_null() {
            let item = &*cursor;
            if !item.ifa_addr.is_null()
                && !item.ifa_netmask.is_null()
                && (*item.ifa_addr).sa_family as i32 == libc::AF_INET
            {
                let address = &*(item.ifa_addr as *const libc::sockaddr_in);
                let mask = &*(item.ifa_netmask as *const libc::sockaddr_in);
                if Ipv4Addr::from(u32::from_be(address.sin_addr.s_addr)) == interface {
                    let bits = u32::from_be(mask.sin_addr.s_addr);
                    let len = bits.leading_ones();
                    if len > 0 && bits == u32::MAX << (32 - len) {
                        result = Some(len as u8);
                        break;
                    }
                }
            }
            cursor = item.ifa_next;
        }
        libc::freeifaddrs(list);
        result
    }
}
#[cfg(not(any(windows, unix)))]
fn system_prefix(_: Ipv4Addr) -> Option<u8> {
    None
}
