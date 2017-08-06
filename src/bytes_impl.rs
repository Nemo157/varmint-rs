use bytes::Buf;

use error::Result;
use parser::Parser;

/// A trait to allow reading integers from a memory buffer that were encoded
/// in the varint format defined by Google's Protocol Buffer standard.
pub trait BufVarInt: Buf {
    /// Read a varint from this object that will fit into a `u64`.
    ///
    /// # Errors
    ///
    /// If the varint read exceeds the space available in a `u64` an error
    /// will be returned. The current position will not be advanced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use varmint::BufVarInt;
    /// use std::io::Cursor;
    /// let mut bytes = Cursor::new(vec![0xB5, 0xFF, 0xAC, 0x02]);
    /// assert_eq!(bytes.get_u64_varint().unwrap(), 0x4B3FB5);
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is no more remaining data in `self`. See
    /// [`try_get_u64_varint`](#fn.try_get_u64_varint) for a non-panicking
    /// variant.
    fn get_u64_varint(&mut self) -> Result<u64>;

    /// Read a varint from this object that will fit into a `usize`.
    ///
    /// # Errors
    ///
    /// If the varint read exceeds the space available in a `usize` an error
    /// will be returned. The current position will not be advanced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use varmint::BufVarInt;
    /// use std::io::Cursor;
    /// let mut bytes = Cursor::new(vec![0xB5, 0xFF, 0xAC, 0x02]);
    /// assert_eq!(bytes.get_usize_varint().unwrap(), 0x4B3FB5);
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is no more remaining data in `self`. See
    /// [`try_get_usize_varint`](#fn.try_get_usize_varint) for a non-panicking
    /// variant.
    fn get_usize_varint(&mut self) -> Result<usize>;

    /// Attempt to read a varint from this object that will fit into a `u64`.
    ///
    /// If this object does not contain a full varint value then this will
    /// return `Ok(None)` and not advance the current position.
    ///
    /// # Errors
    ///
    /// If the varint read exceeds the space available in a `u64` an error
    /// will be returned. The current position will not be advanced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use varmint::BufVarInt;
    /// use std::io::Cursor;
    /// let mut bytes = Cursor::new(vec![0xB5, 0xFF, 0xAC, 0x02]);
    /// assert_eq!(bytes.try_get_u64_varint().unwrap(), Some(0x4B3FB5));
    /// ```
    ///
    /// ```rust
    /// use varmint::BufVarInt;
    /// use std::io::Cursor;
    /// let mut bytes = Cursor::new(vec![]);
    /// assert_eq!(bytes.try_get_u64_varint().unwrap(), None);
    /// ```
    fn try_get_u64_varint(&mut self) -> Result<Option<u64>>;

    /// Attempt to read a varint from this object that will fit into a `usize`.
    ///
    /// If this object does not contain a full varint value then this will
    /// return `Ok(None)` and not advance the current position.
    ///
    /// # Errors
    ///
    /// If the varint read exceeds the space available in a `usize` an error
    /// will be returned. The current position will not be advanced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use varmint::BufVarInt;
    /// use std::io::Cursor;
    /// let mut bytes = Cursor::new(vec![0xB5, 0xFF, 0xAC, 0x02]);
    /// assert_eq!(bytes.try_get_usize_varint().unwrap(), Some(0x4B3FB5));
    /// ```
    ///
    /// ```rust
    /// use varmint::BufVarInt;
    /// use std::io::Cursor;
    /// let mut bytes = Cursor::new(vec![]);
    /// assert_eq!(bytes.try_get_usize_varint().unwrap(), None);
    /// ```
    fn try_get_usize_varint(&mut self) -> Result<Option<usize>>;
}

impl<B: Buf> BufVarInt for B {
    fn get_u64_varint(&mut self) -> Result<u64> {
        let mut parser = Parser::new();
        while !parser.done() {
            parser.push(self.get_u8())?;
        }
        Ok(parser.result())
    }

    fn try_get_u64_varint(&mut self) -> Result<Option<u64>> {
        let mut parser = Parser::new();
        let mut i = 0;
        {
            let bytes = self.bytes();
            while i < bytes.len() && !parser.done() {
                parser.push(bytes[i])?;
                i += 1;
            }
        }
        if parser.done() {
            self.advance(i);
            Ok(Some(parser.result()))
        } else {
            Ok(None)
        }
    }

    fn get_usize_varint(&mut self) -> Result<usize> {
        self.get_u64_varint().map(|u| u as usize)
        // Note: assumes that `usize` is not larger than 64bits, which is the case for every single
        // platform supported by Rust today.
        let val = self.get_u64_varint()?;
        if val <= usize::max_value() as u64 {
            Ok(val as usize)
        } else {
            Err(Error::LengthExceeded)
        }
    }

    fn try_get_usize_varint(&mut self) -> Result<Option<usize>> {
        // Note: assumes that `usize` is not larger than 64bits, which is the case for every single
        // platform supported by Rust today.
        let val = self.try_get_u64_varint()?;
        match val {
            Some(v) => {
                if v <= usize::max_value() as u64 {
                    Ok(Some(v as usize))
                } else {
                    Err(Error::LengthExceeded)
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
    use std::io::Cursor;
    use BufVarInt;

    #[test]
    fn zero() {
        let mut bytes = Cursor::new(vec![0]);
        assert_eq!(bytes.get_u64_varint().unwrap(), 0);
    }

    #[test]
    fn one() {
        let mut bytes = Cursor::new(vec![1]);
        assert_eq!(bytes.get_u64_varint().unwrap(), 1);
    }

    #[test]
    fn some() {
        let mut bytes = Cursor::new(vec![0xAC, 0x02]);
        assert_eq!(bytes.get_u64_varint().unwrap(), 0x12C);
    }

    #[test]
    fn many() {
        let mut bytes = Cursor::new(vec![0xB5, 0xFF, 0xAC, 0x02]);
        assert_eq!(bytes.get_u64_varint().unwrap(), 0x4B3FB5);
    }

    #[test]
    fn half() {
        let mut bytes = Cursor::new(vec![
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0x7F,
        ]);
        assert_eq!(bytes.get_u64_varint().unwrap(), 0x7FFFFFFFFFFFFFFF);
    }

    #[test]
    fn all() {
        let mut bytes = Cursor::new(vec![
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x01,
        ]);
        assert_eq!(bytes.get_u64_varint().unwrap(), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn too_many() {
        let mut bytes = Cursor::new(vec![
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x02,
        ]);
        assert!(bytes.get_u64_varint().is_err());
    }

    #[test]
    fn try_some() {
        let mut bytes = Cursor::new(vec![0xAC, 0x02]);
        assert_eq!(bytes.try_get_u64_varint().unwrap(), Some(0x12C));
    }

    #[test]
    fn try_none() {
        let mut bytes = Cursor::new(vec![]);
        assert_eq!(bytes.try_get_u64_varint().unwrap(), None);
    }
}
