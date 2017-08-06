use error::{Error, Result};

pub(crate) struct Parser {
    done: bool,
    result: u64,
    offset: usize,
}

impl Parser {
    pub(crate) fn new() -> Parser {
        Parser {
            done: false,
            result: 0,
            offset: 0,
        }
    }

    pub(crate) fn push(&mut self, byte: u8) -> Result<()> {
        if self.done {
            panic!("Parser already done")
        }

        if self.offset == 63 {
            self.done = true;
            if byte == 0x01 {
                self.result += 1 << 63;
            } else {
                return Err(Error::LengthExceeded);
            }
        } else {
            self.done = byte & 0x80 == 0;
            self.result += u64::from(byte & 0x7F) << self.offset;
            self.offset += 7;
        }
        Ok(())
    }

    pub(crate) fn done(&self) -> bool {
        self.done
    }

    pub(crate) fn result(self) -> u64 {
        if self.done {
            self.result
        } else {
            panic!("Parser not done")
        }
    }
}
