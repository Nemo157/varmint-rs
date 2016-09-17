use arrayvec::ArrayVec;
use futures::Future;
use std::io as stdio;
use tokio_core::io as tokio;

pub fn write_u64_varint<W: stdio::Write>(w: W, mut val: u64) -> impl Future<Item=W, Error=stdio::Error> {
    let mut bytes = ArrayVec::<[_; 10]>::new();
    while val > 0x7F {
        bytes.push(val as u8 | 0x80);
        val = val >> 7;
    }
    bytes.push(val as u8);
    tokio::write_all(w, bytes).map(|(w, _)| w)
}

#[cfg(target_arch = "x86_64")] // TODO: better cfg detection of this
pub fn write_usize_varint<W: stdio::Write>(w: W, val: usize) -> impl Future<Item=W, Error=stdio::Error> {
    write_u64_varint(w, val as u64)
}

#[cfg(test)]
mod tests {
    use futures::Future;
    use super::write_u64_varint;

    #[test]
    fn zero() {
        let expected: &[u8] = &[0];
        let bytes = write_u64_varint(vec![], 0).wait().unwrap();
        assert_eq!(&bytes[..], expected);
    }

    #[test]
    fn one() {
        let expected: &[u8] = &[1];
        let bytes = write_u64_varint(vec![], 1).wait().unwrap();
        assert_eq!(&bytes[..], expected);
    }

    #[test]
    fn some() {
        let expected: &[u8] = &[0xAC, 0x02];
        let bytes = write_u64_varint(vec![], 0x12C).wait().unwrap();
        assert_eq!(&bytes[..], expected);
    }

    #[test]
    fn many() {
        let expected: &[u8] = &[0xB5, 0xFF, 0xAC, 0x02];
        let bytes = write_u64_varint(vec![], 0x4B3FB5).wait().unwrap();
        assert_eq!(&bytes[..], expected);
    }

    #[test]
    fn half() {
        let expected: &[u8] = &[
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0x7F,
        ];
        let bytes = write_u64_varint(vec![], 0x7FFFFFFFFFFFFFFF).wait().unwrap();
        assert_eq!(&bytes[..], expected);
    }

    #[test]
    fn all() {
        let expected: &[u8] = &[
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x01,
        ];
        let bytes = write_u64_varint(vec![], 0xFFFFFFFFFFFFFFFF).wait().unwrap();
        assert_eq!(&bytes[..], expected);
    }
}
