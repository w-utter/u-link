pub mod socket_state {
    pub struct Initializing;
    pub struct Listening;
}

pub struct Monitor<S> {
    socket: neli::socket::NlSocket,
    _state: S,
}

impl Monitor<socket_state::Initializing> {
    pub fn new() -> std::io::Result<Self> {
        use neli::{consts::socket::NlFamily, socket::NlSocket, utils::Groups};
        let sock = NlSocket::new(NlFamily::Route)?;

        sock.add_mcast_membership(Groups::new_groups(&[libc::RTMGRP_LINK as _]))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "could not register multicast membership `link` to netlink socket"))?;

        Ok(Self {
            socket: sock,
            _state: socket_state::Initializing,
        })
    }

    pub fn set_nonblock(&self) -> std::io::Result<()> {
        self.socket.nonblock()
    }

    pub fn listen(self) -> std::io::Result<Monitor<socket_state::Listening>> {
        use neli::utils::Groups;
        let Self { socket, .. } = self;

        socket.bind(None, Groups::new_bitmask(1))?;

        Ok(Monitor {
            socket,
            _state: socket_state::Listening,
        })
    }
}

impl Monitor<socket_state::Listening> {
    pub fn recv<B: AsMut<[u8]>>(&mut self, buffer: B) -> std::io::Result<usize> {
        self.recv_with_flags(buffer, neli::consts::socket::Msg::empty())
    }

    pub fn recv_with_flags<B: AsMut<[u8]>>(
        &mut self,
        buffer: B,
        flags: neli::consts::socket::Msg,
    ) -> std::io::Result<usize> {
        let (len, _) = self.socket.recv(buffer, flags)?;
        Ok(len)
    }
}

impl<S> std::os::fd::AsRawFd for Monitor<S> {
    fn as_raw_fd(&self) -> i32 {
        self.socket.as_raw_fd()
    }
}
