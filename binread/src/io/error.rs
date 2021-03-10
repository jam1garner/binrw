pub struct Error {
    repr: Repr
}

use core::fmt;

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

#[derive(Debug)]
enum Repr {
    Simple(ErrorKind),
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    Other,
    UnexpectedEof,
}

impl Error {
    pub fn new<A>(kind: ErrorKind, _: A) -> Self {
        Self {
            repr: Repr::Simple(kind)
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Simple(kind) => kind,
        }
    }
}
