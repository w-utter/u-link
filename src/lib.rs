pub mod enumerator;
pub mod packet;
pub(crate) mod socket;
pub use socket::{Monitor, socket_state};

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};

#[derive(Debug)]
pub struct NetworkItf<'a> {
    pub name: Cow<'a, OsStr>,
    pub operstate: OsString,
}

impl<'a> NetworkItf<'a> {
    pub fn from_bytes(buf: &'a [u8]) -> std::io::Result<Self> {
        let raw = crate::packet::NetlinkItfPacket::from_bytes(buf).unwrap();
        let mut name = None;
        let mut operstate = None;

        use neli::consts::rtnl::Ifla;
        for (attr, val) in raw.rt_attrs() {
            match attr {
                Ifla::Operstate => operstate = Some(val),
                Ifla::Ifname => name = Some(val),
                _ => continue,
            }
        }
        let (Some(name), Some(operstate)) = (name, operstate) else {
            panic!();
        };

        let operstate = match *operstate.get(0).unwrap() as _ {
            libc::IF_OPER_UP => "up",
            libc::IF_OPER_DOWN => "down",
            libc::IF_OPER_DORMANT => "dormant",
            libc::IF_OPER_TESTING => "testing",
            libc::IF_OPER_UNKNOWN => "unknown",
            libc::IF_OPER_LOWERLAYERDOWN => "lowerlayerdown",
            libc::IF_OPER_NOTPRESENT => "notpresent",
            _ => todo!(),
        };

        let name = if name.ends_with(b"\0") {
            &name[..name.len() - 1]
        } else {
            name
        };

        Ok(Self {
            // SAFETY: bytes are coming directly from the os
            name: unsafe { OsStr::from_encoded_bytes_unchecked(name) }.into(),
            operstate: operstate.into(),
        })
    }
}
