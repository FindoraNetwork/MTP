// ### About nibble
// Briefly, a nibble is just a hex char, it is desgined for the BranchNode.
//
// For example.
// the key is H160 which contains 20 bytes, the first bytes have u8::MAX branches, in this design, our BranchNode would be like:
//
//
//                          BranchNode
//              /    /     / ......  \     \
//           Node  Node  None      None   None   x 256
//
// this would be a [Node; 256] that contains many many None child here, it wastes and would not be friend with stack.
// the solution here is the hex encode; "1111" is just 15, so for the BranchNode, the chlid have only 16.
// wasting is largely alleviated, but the deepth increase double.
// all this is engineering consideration.
//
// About HP-encode（Hex-Prefix Encoding ）
// beacuse the hex encode used in nibble, one byte become two bytes, it's not good for serialization.
// we use Hp-encode to compress nibbles.
//
// Nibbles used for Leaf node end with 0x10(16).
// 

use std::cmp::min;

use crate::TrieError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Nibbles {
    hex_data: Vec<u8>,
}

impl Nibbles {
    pub(crate) fn from_hex_unchecked(hex_data: Vec<u8>) -> Self {
        Nibbles { hex_data }
    }

    pub fn from_bytes(raw: &[u8], is_leaf: bool) -> Self {
        let mut hex_data = Vec::with_capacity(raw.len() * 2);
        for item in raw.iter() {
            hex_data.push(*item / 16);
            hex_data.push(*item % 16);
        }
        if is_leaf {
            hex_data.push(16);
        }
        Nibbles { hex_data }
    }

    pub fn from_compact(compact: Vec<u8>) -> Result<Self, TrieError> {
        let mut hex = vec![];
        let flag = compact[0];

        let mut is_leaf = false;
        match flag >> 4 {
            0x0 => {}
            0x1 => hex.push(flag % 16),
            0x2 => is_leaf = true,
            0x3 => {
                is_leaf = true;
                hex.push(flag % 16);
            }
            _ => return Err(TrieError::InvalidData),
        };

        for item in &compact[1..] {
            hex.push(item / 16);
            hex.push(item % 16);
        }
        if is_leaf {
            hex.push(16);
        }

        Ok(Nibbles { hex_data: hex })
    }

    pub fn is_leaf(&self) -> bool {
        *self.hex_data.last().unwrap() == 16
    }

    pub fn encode_compact(&self) -> Vec<u8> {
        let mut compact = vec![];
        let is_leaf = self.is_leaf();
        let mut hex = if is_leaf {
            &self.hex_data[..self.hex_data.len() - 1]
        } else {
            &self.hex_data
        };
        // node type    path length    |    prefix    hexchar
        // --------------------------------------------------
        // extension    even           |    0000      0x0
        // extension    odd            |    0001      0x1
        // leaf         even           |    0010      0x2
        // leaf         odd            |    0011      0x3
        let v = if hex.len() % 2 == 1 {
            let v = 0x10 + hex[0];
            hex = &hex[1..];
            v
        } else {
            0x00
        };

        compact.push(v + if is_leaf { 0x20 } else { 0x00 });
        for i in 0..(hex.len() / 2) {
            compact.push((hex[i * 2] * 16) + (hex[i * 2 + 1]));
        }

        compact
    }

    pub fn encode_raw(&self) -> (Vec<u8>, bool) {
        let mut raw = vec![];
        let is_leaf = self.is_leaf();
        let hex = if is_leaf {
            &self.hex_data[..self.hex_data.len() - 1]
        } else {
            &self.hex_data[..]
        };

        for i in 0..(hex.len() / 2) {
            raw.push((hex[i * 2] * 16) + (hex[i * 2 + 1]));
        }

        (raw, is_leaf)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.hex_data.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn hex_at(&self, i: usize) -> usize {
        self.hex_data[i] as usize
    }

    pub fn common_prefix(&self, other_partial: &Nibbles) -> usize {
        let s = min(self.len(), other_partial.len());
        let mut i = 0usize;
        while i < s {
            if self.hex_at(i) != other_partial.hex_at(i) {
                break;
            }
            i += 1;
        }
        i
    }

    #[inline(always)]
    pub fn offset(&self, index: usize) -> Nibbles {
        self.slice(index, self.hex_data.len())
    }

    #[inline(always)]
    pub fn slice(&self, start: usize, end: usize) -> Nibbles {
        Nibbles::from_hex_unchecked(self.hex_data[start..end].to_owned())
    }

    #[inline(always)]
    pub fn get_data(&self) -> &[u8] {
        &self.hex_data
    }

    pub fn join(&self, b: &Nibbles) -> Nibbles {
        let len = self.get_data().len() + b.get_data().len();
        let mut hex_data = Vec::with_capacity(len);
        hex_data.extend_from_slice(self.get_data());
        hex_data.extend_from_slice(b.get_data());
        Nibbles::from_hex_unchecked(hex_data)
    }

    pub fn extend(&mut self, b: &Nibbles) {
        self.hex_data.extend_from_slice(b.get_data());
    }

    pub fn truncate(&mut self, len: usize) {
        self.hex_data.truncate(len);
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.hex_data.pop()
    }

    pub fn push(&mut self, e: u8) {
        self.hex_data.push(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nibble() {
        let n = Nibbles::from_bytes(b"key1", true);
        let compact = n.encode_compact();
        let n2 = Nibbles::from_compact(compact).unwrap();
        let (raw, is_leaf) = n2.encode_raw();
        assert!(is_leaf);
        assert_eq!(raw, b"key1");
    }
}
