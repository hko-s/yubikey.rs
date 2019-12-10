//! Cardholder Capability Container (CCC) ID Support

// Adapted from yubico-piv-tool:
// <https://github.com/Yubico/yubico-piv-tool/>
//
// Copyright (c) 2014-2016 Yubico AB
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//   * Redistributions of source code must retain the above copyright
//     notice, this list of conditions and the following disclaimer.
//
//   * Redistributions in binary form must reproduce the above
//     copyright notice, this list of conditions and the following
//     disclaimer in the documentation and/or other materials provided
//     with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::{error::Error, yubikey::YubiKey};
use getrandom::getrandom;
use std::fmt::{self, Debug, Display};
use subtle_encoding::hex;

/// CCCID size
pub const CCCID_SIZE: usize = 14;

/// CCC size
pub const CCC_SIZE: usize = 51;

/// CCCID offset
const CCC_ID_OFFS: usize = 9;

/// CCC Object ID
const OBJ_CAPABILITY: u32 = 0x005f_c107;

/// Cardholder Capability Container (CCC) Template
///
/// f0: Card Identifier
///
///  - 0xa000000116 == GSC-IS RID
///  - 0xff == Manufacturer ID (dummy)
///  - 0x02 == Card type (javaCard)
///  - next 14 bytes: card ID
const CCC_TMPL: &[u8] = &[
    0xf0, 0x15, 0xa0, 0x00, 0x00, 0x01, 0x16, 0xff, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf1, 0x01, 0x21, 0xf2, 0x01, 0x21, 0xf3, 0x00, 0xf4,
    0x01, 0x00, 0xf5, 0x01, 0x10, 0xf6, 0x00, 0xf7, 0x00, 0xfa, 0x00, 0xfb, 0x00, 0xfc, 0x00, 0xfd,
    0x00, 0xfe, 0x00,
];

/// Cardholder Capability Container (CCC) Identifier Card ID
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CardId(pub [u8; CCCID_SIZE]);

impl CardId {
    /// Generate a random CCC Card ID
    pub fn generate() -> Result<Self, Error> {
        let mut id = [0u8; CCCID_SIZE];
        getrandom(&mut id).map_err(|_| Error::RandomnessError)?;
        Ok(Self(id))
    }
}

/// Cardholder Capability Container (CCC) Identifier
#[derive(Copy, Clone)]
pub struct CCC(pub [u8; CCC_SIZE]);

impl CCC {
    /// Return CardId component of CCC
    pub fn cccid(&self) -> Result<CardId, Error> {
        let mut cccid = [0u8; CCCID_SIZE];
        cccid.copy_from_slice(&self.0[CCC_ID_OFFS..(CCC_ID_OFFS + CCCID_SIZE)]);
        Ok(CardId(cccid))
    }

    /// Get Cardholder Capability Container (CCC) ID
    pub fn get(yubikey: &mut YubiKey) -> Result<Self, Error> {
        let txn = yubikey.begin_transaction()?;
        let response = txn.fetch_object(OBJ_CAPABILITY)?;

        if response.len() != CCC_TMPL.len() {
            return Err(Error::GenericError);
        }

        let mut ccc = [0u8; CCC_SIZE];
        ccc.copy_from_slice(&response[0..CCC_SIZE]);
        Ok(Self(ccc))
    }

    /// Get Cardholder Capability Container (CCC) ID
    #[cfg(feature = "untested")]
    pub fn set(&self, yubikey: &mut YubiKey) -> Result<(), Error> {
        let mut buf = CCC_TMPL.to_vec();
        buf[0..self.0.len()].copy_from_slice(&self.0);

        let txn = yubikey.begin_transaction()?;
        txn.save_object(OBJ_CAPABILITY, &buf)
    }
}

impl Debug for CCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CCC({:?})", &self.0[..])
    }
}

impl Display for CCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            String::from_utf8(hex::encode(&self.0[..])).unwrap()
        )
    }
}
