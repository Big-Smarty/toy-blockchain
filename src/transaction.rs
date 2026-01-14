use std::fmt::Display;

use chrono::{DateTime, Utc};
use sha1::Digest;

const MATRIKEL_NUMMER: u64 = 285765;
const NAME: &str = "Daniel Budeanu";
const RECIPIENTS: [&str; 3] = ["Alice", "Bob", "Carol"];
const VALUES: [u64; 3] = [69, 420, 67];

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

    // TODO: pad string such that the transaction has its own two aligned words which can simply be overwritten
    pub fn nonce_offsets(&self) -> (u32, u32) {
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
        let pre_nonce_string = format!(
            "{transaction_string}{length}{iv0:08x}{iv1:08x}{iv2:08x}{iv3:08x}{iv4:08x}{iv5:08x}{iv6:08x}{iv7:08x}",
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
        let byte_offset = pre_nonce_string.len() % 4;

        (word_idx as u32, byte_offset as u32)
    }
}

// TODO: pad string such that the transaction has its own two aligned words which can simply be overwritten
impl Display for Transaction {
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
        let transaction_string = format!(
            "{transaction_string}{length}{iv0:08x}{iv1:08x}{iv2:08x}{iv3:08x}{iv4:08x}{iv5:08x}{iv6:08x}{iv7:08x}{nonce:08x}",
            iv0 = self.iv[0],
            iv1 = self.iv[1],
            iv2 = self.iv[2],
            iv3 = self.iv[3],
            iv4 = self.iv[4],
            iv5 = self.iv[5],
            iv6 = self.iv[6],
            iv7 = self.iv[7],
            nonce = self.nonce,
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
