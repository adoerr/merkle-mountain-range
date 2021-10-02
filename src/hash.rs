// Copyright (C) 2021 Andreas Doerr
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Hash type

use core::{
    cmp::min,
    convert::AsRef,
    fmt::{self, Write},
};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

use blake2::{Blake2b, Digest};
use codec::{Decode, Encode};

use crate::{Error, String, Vec};

#[cfg(test)]
#[path = "hash_tests.rs"]
mod tests;

macro_rules! to_hex {
    ($bytes:expr) => {{
        let mut s = String::with_capacity(64);

        for b in $bytes {
            write!(&mut s, "{:02x}", b)?
        }

        Ok(s)
    }};
}

/// Generic hash type which should be compatible with most hashes used
/// within the blockchain domain.
#[derive(Copy, Clone, PartialEq, Encode, Decode)]
pub struct Hash([u8; 32]);

/// A hash consisting of all zeros.
pub const ZERO_HASH: Hash = Hash([0; 32]);

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const DISP_SIZE: usize = 12;

        let hex = to_hex!(&self.0)?;
        write!(f, "{}", &hex[..DISP_SIZE])
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Hash {
    /// 32 byte hash
    pub const LEN: usize = 32;

    /// Return a hash initialized from `v`.
    ///
    /// At most, up to [`Hash::LEN`] bytes will be copied from `v`. If `v` has less
    /// than [`Hash::LEN`] bytes, the hash will be padded with 0's from left to right.
    pub fn from_vec(v: &[u8]) -> Hash {
        let mut h = [0; Hash::LEN];
        let sz = min(v.len(), Hash::LEN);
        h[..sz].copy_from_slice(&v[..sz]);
        Hash(h)
    }

    /// Retrun a hash initialized from string `hex`.
    ///
    /// An error is returned, if `hex` is not a well-formed hex string like `"0xcafe"`.
    pub fn from_hex(hex: &str) -> Result<Hash, Error> {
        match parse_hex(hex) {
            Ok(v) => Ok(Hash::from_vec(&v)),
            Err(s) => Err(Error::InvalidHexString(s)),
        }
    }
}

fn parse_hex(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim().trim_start_matches("0x");

    if hex.len() % 2 != 0 {
        Err(hex.to_string())
    } else {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| hex.to_string()))
            .collect()
    }
}

/// Types with a canonical hash
pub trait Hashable {
    fn hash(&self) -> Hash;
}

impl Hashable for Vec<u8> {
    fn hash(&self) -> Hash {
        let mut h = Blake2b::new();
        h.update(self);
        let v = h.finalize();
        Hash::from_vec(&v)
    }
}

impl Hashable for u32 {
    fn hash(&self) -> Hash {
        let mut h = Blake2b::new();
        h.update(self.to_le_bytes());
        let v = h.finalize();
        Hash::from_vec(&v)
    }
}

impl Hashable for u64 {
    fn hash(&self) -> Hash {
        let mut h = Blake2b::new();
        h.update(self.to_le_bytes());
        let v = h.finalize();
        Hash::from_vec(&v)
    }
}

impl Hashable for Hash {
    /// Return the hash, without hashing again.
    fn hash(&self) -> Hash {
        *self
    }
}

impl<A, B> Hashable for (A, B)
where
    A: Hashable,
    B: Hashable,
{
    fn hash(&self) -> Hash {
        let mut h = Blake2b::new();
        h.update(self.0.hash());
        h.update(self.1.hash());
        let v = h.finalize();
        Hash::from_vec(&v)
    }
}

/// Return the hash of `idx` and `hash`.
///
/// This function is used to avoid collisions among leaf data hashes themselves.
pub fn hash_with_index(idx: u64, hash: &Hash) -> Hash {
    let mut h = Blake2b::new();
    h.update(idx.to_le_bytes());
    h.update(hash);
    let v = h.finalize();
    Hash::from_vec(&v)
}
