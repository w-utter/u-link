pub struct NetlinkItfPacket<'a> {
    pub nl_header: &'a NlMsgHdr,
    pub info: &'a IfInfoMsg,
    payload: &'a [u8],
}

impl core::fmt::Debug for NetlinkItfPacket<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct DebugAttrs<'a> {
            inner: &'a NetlinkItfPacket<'a>,
        }

        impl core::fmt::Debug for DebugAttrs<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_list().entries(self.inner.rt_attrs()).finish()
            }
        }

        f.debug_struct("NetlinkItfPacket")
            .field("nl_header", self.nl_header)
            .field("info", self.info)
            .field("attrs", &DebugAttrs { inner: self })
            .finish()
    }
}

impl<'a> NetlinkItfPacket<'a> {
    pub fn from_bytes(buf: &'a [u8]) -> std::io::Result<Self> {
        const HEADER_LEN: usize = core::mem::size_of::<NlMsgHdr>();
        const INFO_LEN: usize = core::mem::size_of::<IfInfoMsg>();
        if buf.len() < HEADER_LEN + INFO_LEN {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "packet too small"));
        }

        let header = bytemuck::from_bytes::<NlMsgHdr>(&buf[..HEADER_LEN]);

        let info = bytemuck::from_bytes::<IfInfoMsg>(&buf[HEADER_LEN..HEADER_LEN + INFO_LEN]);

        Ok(Self {
            nl_header: header,
            info,
            payload: &buf[HEADER_LEN + INFO_LEN..],
        })
    }
}

impl<'a> NetlinkItfPacket<'a> {
    pub fn rt_attrs(&self) -> impl Iterator<Item = (neli::consts::rtnl::Ifla, &'a [u8])> {
        RtAttrIter {
            buf: self.payload,
            offset: 0,
        }
    }
}

pub struct RtAttrIter<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for RtAttrIter<'a> {
    type Item = (neli::consts::rtnl::Ifla, &'a [u8]);
    fn next(&mut self) -> Option<Self::Item> {
        let Some(bytes) = self
            .buf
            .get(self.offset..self.offset + core::mem::size_of::<RtAttr>())
        else {
            return None;
        };

        let attr = bytemuck::from_bytes::<RtAttr>(bytes);

        if (attr.len as usize) < core::mem::size_of::<RtAttr>()
            || self.offset + (attr.len as usize) > self.buf.len()
        {
            self.offset = self.buf.len();
            return None;
        }

        let payload = &self.buf
            [self.offset + core::mem::size_of::<RtAttr>()..self.offset + attr.len as usize];

        // align to 4 byte boundary
        const RT_ALIGN: usize = 4;

        self.offset += (attr.len as usize + (RT_ALIGN - 1)) & !(RT_ALIGN - 1);
        Some((attr.ty(), payload))
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct NlMsgHdr {
    len: u32,
    ty: u16,
    flags: u16,
    seq: u32,
    pid: u32,
}

impl NlMsgHdr {
    pub fn ty(&self) -> neli::consts::rtnl::Rtm {
        neli::consts::rtnl::Rtm::from(self.ty)
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct IfInfoMsg {
    family: u8,
    _padding: u8,
    ty: u16,
    index: libc::c_int,
    flags: libc::c_uint,
    change: libc::c_uint,
}

impl IfInfoMsg {
    pub fn addr_family(&self) -> neli::consts::rtnl::RtAddrFamily {
        neli::consts::rtnl::RtAddrFamily::from(self.family)
    }

    pub fn ty(&self) -> neli::consts::rtnl::Arphrd {
        neli::consts::rtnl::Arphrd::from(self.ty)
    }

    pub fn flags(&self) -> neli::consts::rtnl::Iff {
        neli::consts::rtnl::Iff::from(self.flags)
    }

    pub fn change(&self) -> neli::consts::rtnl::Iff {
        neli::consts::rtnl::Iff::from(self.change)
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct RtAttr {
    len: u16,
    ty: u16,
}

impl RtAttr {
    pub fn ty(&self) -> neli::consts::rtnl::Ifla {
        neli::consts::rtnl::Ifla::from(self.ty)
    }
}
