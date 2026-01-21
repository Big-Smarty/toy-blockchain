use std::fmt::Display;

use chrono::{DateTime, Utc};
use sha1::Digest;

use crate::util;

const MATRIKEL_NUMMER: u64 = 285765;
const NAME: &str = "Daniel Budeanu";
const RECIPIENTS: [&str; 3] = ["Alice", "Bob", "Carol"];
const VALUES: [u64; 3] = [69, 420, 67];

#[derive(Clone)]
pub struct Transaction {
    pub(crate) transaction_number: u64,
    pub(crate) name: &'static str,
    pub(crate) datetime: DateTime<Utc>,
    pub(crate) recipient: &'static str,
    pub(crate) value: u64,
    pub(crate) iv: [u32; 8],
    pub(crate) nonce: u64,
}

impl Transaction {
    pub fn new(old_transaction: Transaction, hash: [u32; 8]) -> Self {
        let generation = old_transaction.transaction_number as usize + 1 - MATRIKEL_NUMMER as usize;
        Self {
            transaction_number: generation as u64 + MATRIKEL_NUMMER,
            name: NAME,
            datetime: Utc::now(),
            recipient: RECIPIENTS[generation],
            value: VALUES[generation],
            iv: hash.clone(),
            nonce: u64::MAX,
        }
    }

    pub fn _with_nonce(&self, nonce: u64) -> Self {
        Self {
            transaction_number: self.transaction_number,
            name: self.name,
            datetime: self.datetime,
            recipient: self.recipient,
            value: self.value,
            iv: self.iv,
            nonce,
        }
    }

    // NOTE: this SHOULD give the correct nonce offset. might have to look at it again if things start going wrong.
    pub fn nonce_offset(&self) -> u32 {
        let mut hasher = sha1::Sha1::new();
        let transaction_string = format!(
            "{transaction_number}{name}{datetime}{recipient}{value}",
            transaction_number = self.transaction_number,
            name = {
                hasher.update(self.name);
                hasher.finalize()[..]
                    .iter()
                    .map(|x| format!("{x:x}").chars().nth(0).unwrap())
                    .collect::<String>()
            },
            datetime = self.datetime.to_rfc3339(),
            recipient = self.recipient,
            value = self.value,
        );
        let length = transaction_string.len();
        let pad_length = util::calculate_manual_pad(length);
        let pre_nonce_string = format!(
            "{transaction_string:<pad_length$}{length:16}{iv0:08x}{iv1:08x}{iv2:08x}{iv3:08x}{iv4:08x}{iv5:08x}{iv6:08x}{iv7:08x}",
            iv0 = self.iv[0],
            iv1 = self.iv[1],
            iv2 = self.iv[2],
            iv3 = self.iv[3],
            iv4 = self.iv[4],
            iv5 = self.iv[5],
            iv6 = self.iv[6],
            iv7 = self.iv[7],
        );
        let word_idx = pre_nonce_string.len() / 4;
        word_idx as u32
    }
}

impl Display for Transaction {
    // NOTE: this SHOULD give the properly padded string. might have to look at it again if things start going wrong.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hasher = sha1::Sha1::new();
        let transaction_string = format!(
            "{transaction_number}{name}{datetime}{recipient}{value}",
            transaction_number = self.transaction_number,
            name = {
                hasher.update(self.name);
                hasher.finalize()[..]
                    .iter()
                    .map(|x| format!("{x:x}").chars().nth(0).unwrap())
                    .collect::<String>()
            },
            datetime = self.datetime.to_rfc3339(),
            recipient = self.recipient,
            value = self.value,
        );
        let length = transaction_string.len();
        let pad_length = util::calculate_manual_pad(length);
        let transaction_string = format!(
            "{transaction_string:<pad_length$}{length:<16}{iv0:08x}{iv1:08x}{iv2:08x}{iv3:08x}{iv4:08x}{iv5:08x}{iv6:08x}{iv7:08x}{nonce}",
            iv0 = self.iv[0],
            iv1 = self.iv[1],
            iv2 = self.iv[2],
            iv3 = self.iv[3],
            iv4 = self.iv[4],
            iv5 = self.iv[5],
            iv6 = self.iv[6],
            iv7 = self.iv[7],
            nonce = util::nonce_to_raw_string(self.nonce),
        );
        write!(f, "{transaction_string}")
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            transaction_number: MATRIKEL_NUMMER,
            name: NAME,
            datetime: Utc::now(),
            recipient: RECIPIENTS[0],
            value: VALUES[0],
            iv: [0, 0, 0, 0, 0, 0, 0, 0],
            nonce: u64::MAX,
        }
    }
}
