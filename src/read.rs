use std::io::{ Read, Error, ErrorKind };
use arrayvec::ArrayVec;
use futures::{ Future, Poll, Async };

pub fn read_u64_varint<R: Read>(reader: R) -> impl Future<Item=(R, u64), Error=Error> {
    TryReadU64Varint {
        reader: Some(reader),
        buffer: ArrayVec::new(),
        allow_none: false,
    }.map(|r| r.unwrap())
}

/// Returns None if EOF on the first byte
pub fn try_read_u64_varint<R: Read>(reader: R) -> impl Future<Item=Option<(R, u64)>, Error=Error> {
    TryReadU64Varint {
        reader: Some(reader),
        buffer: ArrayVec::new(),
        allow_none: true,
    }
}

#[cfg(target_arch = "x86_64")] // TODO: better cfg detection of this
pub fn read_usize_varint<R: Read>(reader: R) -> impl Future<Item=(R, usize), Error=Error> {
    read_u64_varint(reader).map(|(r, u)| (r, u as usize))
}

#[cfg(target_arch = "x86_64")] // TODO: better cfg detection of this
/// Returns None if EOF on the first byte
pub fn try_read_usize_varint<R: Read>(reader: R) -> impl Future<Item=Option<(R, usize)>, Error=Error> {
    try_read_u64_varint(reader).map(|r| r.map(|(r, u)| (r, u as usize)))
}

struct TryReadU64Varint<R: Read> {
    reader: Option<R>,
    buffer: ArrayVec<[u8; 10]>,
    allow_none: bool,
}

impl<R: Read> Future for TryReadU64Varint<R> {
    type Item = Option<(R, u64)>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<(R, u64)>, Error> {
        let mut reader = self.reader.take().expect("poll a TryReadU64Varint after it's done");

        while self.buffer.len() <= 9 && (self.buffer.is_empty() || *self.buffer.last().unwrap() >= 0x80) {
            let mut b = [0];
            match reader.read_exact(&mut b) {
                Ok(()) => {
                    self.buffer.push(b[0]);
                }
                Err(err) => return match err.kind() {
                    ErrorKind::WouldBlock => {
                        self.reader = Some(reader);
                        Ok(Async::NotReady)
                    }
                    ErrorKind::UnexpectedEof => {
                        if self.allow_none && self.buffer.is_empty() {
                            Ok(Async::Ready(None))
                        } else {
                            Err(err)
                        }
                    }
                    _ => Err(err)
                }
            }
        }

        if self.buffer.len() == 10 && *self.buffer.last().unwrap() > 0x01 {
            return Err(Error::new(ErrorKind::Other, "varint exceeded 64 bits long"));
        }

        let result = self.buffer
            .iter()
            .enumerate()
            .map(|(i, b)| (((b & 0x7F) as u64) << (7 * i)))
            .sum();

        return Ok(Async::Ready(Some((reader, result))));
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use futures::Future;
    use super::{ read_u64_varint, try_read_u64_varint };

    fn r<A, B, E: fmt::Debug, F: Future<Item=(A, B), Error=E>>(f: F) -> B {
        f.wait().unwrap().1
    }

    fn ro<A, B, E: fmt::Debug, F: Future<Item=Option<(A, B)>, Error=E>>(f: F) -> Option<B> {
        f.wait().unwrap().map(|(_, u)| u)
    }

    #[test]
    fn zero() {
        let bytes: &[u8] = &[0];
        assert_eq!(r(read_u64_varint(bytes)), 0);
    }

    #[test]
    fn one() {
        let bytes: &[u8] = &[1];
        assert_eq!(r(read_u64_varint(bytes)), 1);
    }

    #[test]
    fn some() {
        let bytes: &[u8] = &[0xAC, 0x02];
        assert_eq!(r(read_u64_varint(bytes)), 0x12C);
    }

    #[test]
    fn many() {
        let bytes: &[u8] = &[0xB5, 0xFF, 0xAC, 0x02];
        assert_eq!(r(read_u64_varint(bytes)), 0x4B3FB5);
    }

    #[test]
    fn half() {
        let bytes: &[u8] = &[
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0x7F,
        ];
        assert_eq!(r(read_u64_varint(bytes)), 0x7FFFFFFFFFFFFFFF);
    }

    #[test]
    fn all() {
        let bytes: &[u8] = &[
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x01,
        ];
        assert_eq!(r(read_u64_varint(bytes)), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn too_many() {
        let bytes: &[u8] = &[
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x02,
        ];
        assert!(read_u64_varint(bytes).wait().is_err());
    }

    #[test]
    fn try_some() {
        let bytes: &[u8] = &[0xAC, 0x02];
        assert_eq!(ro(try_read_u64_varint(bytes)), Some(0x12C));
    }

    #[test]
    fn try_none() {
        let bytes: &[u8] = &[];
        assert_eq!(ro(try_read_u64_varint(bytes)), None);
    }
}
