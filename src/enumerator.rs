use crate::NetworkItf;
use std::ffi::OsString;

pub enum ResIter<T, E> {
    Err(std::iter::Once<E>),
    Ok(T),
}

impl<U, E, T: Iterator<Item = Result<U, E>>> Iterator for ResIter<T, E> {
    type Item = Result<U, E>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Err(e) => e.next().map(|e| Err(e)),
            Self::Ok(o) => o.next(),
        }
    }
}

pub fn enumerate() -> impl Iterator<Item = std::io::Result<NetworkItf<'static>>> {
    NetworkItfIter::new()
}

pub struct NetworkItfIter {
    dir: std::fs::ReadDir,
}

impl NetworkItfIter {
    pub(crate) fn new() -> ResIter<Self, std::io::Error> {
        match std::fs::read_dir("/sys/class/net/") {
            Err(e) => ResIter::Err(std::iter::once(e)),
            Ok(dir) => ResIter::Ok(NetworkItfIter { dir }),
        }
    }
}

macro_rules! maybe_err {
    ($e:expr) => {
        match $e {
            Err(e) => return Some(Err(e)),
            Ok(o) => o,
        }
    };
}

impl Iterator for NetworkItfIter {
    type Item = std::io::Result<NetworkItf<'static>>;
    fn next(&mut self) -> Option<Self::Item> {
        let entry = maybe_err!(self.dir.next()?);

        let name = entry.file_name();

        let mut path = entry.path();
        path.push("operstate");

        let mut operstate = maybe_err!(std::fs::read(&path));

        if operstate.ends_with(b"\n") {
            operstate.truncate(operstate.len() - 1);
        }

        Some(Ok(NetworkItf {
            name: name.into(),
            // SAFETY: its coming directly from the os
            operstate: unsafe { OsString::from_encoded_bytes_unchecked(operstate) },
        }))
    }
}
