use std::io;

use parser::Parser;

trait ReadHelper {
    fn read_u8(&mut self) -> io::Result<u8>;
    fn try_read_u8(&mut self) -> io::Result<Option<u8>>;
}

/// A trait to allow reading integers from a byte-oriented source that were
/// encoded in the varint format defined by Google's Protocol Buffer standard.
pub trait ReadVarInt {
    /// Read a varint from this object that will fit into a `u64`.
    ///
    /// # Errors
    ///
    /// If there is an error returned by the underlying source then that will
    /// be propagated up, in that case the number of bytes that will have been
    /// read from it is undefined.
    ///
    /// If the varint read exceeds the space available in a `u64` an error will
    /// be returned and the number of bytes read from the underlying source is
    /// undefined.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use varmint::ReadVarInt;
    /// let mut bytes: &[u8] = &[0xB5, 0xFF, 0xAC, 0x02];
    /// assert_eq!(bytes.read_u64_varint().unwrap(), 0x4B3FB5);
    /// ```
    fn read_u64_varint(&mut self) -> io::Result<u64>;

    /// Read a varint from this object that will fit into a `usize`.
    ///
    /// # Errors
    ///
    /// If there is an error returned by the underlying source then that will
    /// be propagated up, in that case the number of bytes that will have been
    /// read from it is undefined.
    ///
    /// If the varint read exceeds the space available in a `usize` an error
    /// will be returned and the number of bytes read from the underlying
    /// source is undefined.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use varmint::ReadVarInt;
    /// let mut bytes: &[u8] = &[0xB5, 0xFF, 0xAC, 0x02];
    /// assert_eq!(bytes.read_usize_varint().unwrap(), 0x4B3FB5);
    /// ```
    fn read_usize_varint(&mut self) -> io::Result<usize>;

    /// Attempt to read a varint from this object that will fit into a `u64`.
    ///
    /// If this object has no more data available then this will return
    /// `Ok(None)`, if however there is a valid prefix for a varint read then
    /// this object runs out of data before completing it that will be
    /// propagated as a normal error.
    ///
    /// # Errors
    ///
    /// If there is an error returned by the underlying source then that will
    /// be propagated up, in that case the number of bytes that will have been
    /// read from it is undefined.
    ///
    /// If the varint read exceeds the space available in a `u64` an error will
    /// be returned and the number of bytes read from the underlying source is
    /// undefined.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use varmint::ReadVarInt;
    /// let mut bytes: &[u8] = &[0xB5, 0xFF, 0xAC, 0x02];
    /// assert_eq!(bytes.try_read_u64_varint().unwrap(), Some(0x4B3FB5));
    /// ```
    ///
    /// ```rust
    /// use varmint::ReadVarInt;
    /// let mut bytes: &[u8] = &[];
    /// assert_eq!(bytes.try_read_u64_varint().unwrap(), None);
    /// ```
    fn try_read_u64_varint(&mut self) -> io::Result<Option<u64>>;

    /// Attempt to read a varint from this object that will fit into a `usize`.
    ///
    /// If this object has no more data available then this will return
    /// `Ok(None)`, if however there is a valid prefix for a varint read then
    /// this object runs out of data before completing it that will be
    /// propagated as a normal error.
    ///
    /// # Errors
    ///
    /// If there is an error returned by the underlying source then that will
    /// be propagated up, in that case the number of bytes that will have been
    /// read from it is undefined.
    ///
    /// If the varint read exceeds the space available in a `usize` an error
    /// will be returned and the number of bytes read from the underlying
    /// source is undefined.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use varmint::ReadVarInt;
    /// let mut bytes: &[u8] = &[0xB5, 0xFF, 0xAC, 0x02];
    /// assert_eq!(bytes.try_read_usize_varint().unwrap(), Some(0x4B3FB5));
    /// ```
    ///
    /// ```rust
    /// use varmint::ReadVarInt;
    /// let mut bytes: &[u8] = &[];
    /// assert_eq!(bytes.try_read_usize_varint().unwrap(), None);
    /// ```
    fn try_read_usize_varint(&mut self) -> io::Result<Option<usize>>;
}

impl<R: io::Read> ReadHelper for R {
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buffer = [0];
        try!(self.read_exact(&mut buffer));
        Ok(buffer[0])
    }

    fn try_read_u8(&mut self) -> io::Result<Option<u8>> {
        let mut buffer = [0];
        if try!(self.read(&mut buffer)) == 1 {
            Ok(Some(buffer[0]))
        } else {
            Ok(None)
        }
    }
}

fn other<E: ::std::error::Error + Send + Sync + 'static>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

impl<R: io::Read> ReadVarInt for R {
    fn read_u64_varint(&mut self) -> io::Result<u64> {
        let mut parser = Parser::new();
        while !parser.done() {
            parser.push(self.read_u8()?).map_err(other)?;
        }
        Ok(parser.result())
    }

    fn try_read_u64_varint(&mut self) -> io::Result<Option<u64>> {
        let mut parser = Parser::new();
        while !parser.done() {
            if let Some(byte) = self.try_read_u8()? {
                parser.push(byte).map_err(other)?;
            } else {
                return Ok(None);
            }
        }
        Ok(Some(parser.result()))
    }

    fn read_usize_varint(&mut self) -> io::Result<usize> {
        // Note: assumes that `usize` is not larger than 64bits, which is the case for every single
        // platform supported by Rust today.
        let val = self.read_u64_varint()?;
        if val <= usize::max_value() as u64 {
            Ok(val as usize)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "varint exceeded usize maximum bits"))
        }
    }

    fn try_read_usize_varint(&mut self) -> io::Result<Option<usize>> {
        // Note: assumes that `usize` is not larger than 64bits, which is the case for every single
        // platform supported by Rust today.
        let val = self.try_read_u64_varint()?;
        match val {
            Some(v) => {
                if v <= usize::max_value() as u64 {
                    Ok(Some(v as usize))
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "varint exceeded usize maximum bits"))
                }
            }
            None => {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ReadVarInt;

    #[test]
    fn zero() {
        let mut bytes: &[u8] = &[0];
        assert_eq!(bytes.read_u64_varint().unwrap(), 0);
    }

    #[test]
    fn one() {
        let mut bytes: &[u8] = &[1];
        assert_eq!(bytes.read_u64_varint().unwrap(), 1);
    }

    #[test]
    fn some() {
        let mut bytes: &[u8] = &[0xAC, 0x02];
        assert_eq!(bytes.read_u64_varint().unwrap(), 0x12C);
    }

    #[test]
    fn many() {
        let mut bytes: &[u8] = &[0xB5, 0xFF, 0xAC, 0x02];
        assert_eq!(bytes.read_u64_varint().unwrap(), 0x4B3FB5);
    }

    #[test]
    fn half() {
        let mut bytes: &[u8] = &[
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0x7F,
        ];
        assert_eq!(bytes.read_u64_varint().unwrap(), 0x7FFFFFFFFFFFFFFF);
    }

    #[test]
    fn all() {
        let mut bytes: &[u8] = &[
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x01,
        ];
        assert_eq!(bytes.read_u64_varint().unwrap(), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn too_many() {
        let mut bytes: &[u8] = &[
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x02,
        ];
        assert!(bytes.read_u64_varint().is_err());
    }

    #[test]
    fn try_some() {
        let mut bytes: &[u8] = &[0xAC, 0x02];
        assert_eq!(bytes.try_read_u64_varint().unwrap(), Some(0x12C));
    }

    #[test]
    fn try_none() {
        let mut bytes: &[u8] = &[];
        assert_eq!(bytes.try_read_u64_varint().unwrap(), None);
    }
}
