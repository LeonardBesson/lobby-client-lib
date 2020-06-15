use crate::net::Message;
use bytes::Bytes;
use std::borrow::Borrow;
use std::ops::Deref;

pub struct ByteBuffer(Bytes);

impl ByteBuffer {
    pub fn skip(&mut self, len: usize) {
        if len <= self.len() {
            drop(self.0.split_to(len));
        }
    }
}

impl From<Bytes> for ByteBuffer {
    fn from(bytes: Bytes) -> Self {
        ByteBuffer(bytes)
    }
}

impl From<Vec<u8>> for ByteBuffer {
    fn from(vec: Vec<u8>) -> Self {
        ByteBuffer(vec.into())
    }
}

impl Deref for ByteBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.0.deref()
    }
}

impl AsRef<[u8]> for ByteBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Borrow<[u8]> for ByteBuffer {
    fn borrow(&self) -> &[u8] {
        self.0.borrow()
    }
}

#[cfg(tests)]
mod tests {
    use crate::utils::byte_buffer::ByteBuffer;
    use bytes::Bytes;

    #[test]
    fn skip() {
        let mut buffer = ByteBuffer(Bytes::from(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
        buffer.skip(4);

        assert_eq!(&buffer[..], &[5, 6, 7, 8, 9, 10])
    }
}
