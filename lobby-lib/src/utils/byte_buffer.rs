use std::borrow::Borrow;
use std::ops::Deref;

use bytes::Bytes;

use crate::net::Message;

pub struct ByteBuffer(Bytes);

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
