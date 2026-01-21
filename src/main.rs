mod bruteforce;
mod context;
mod push_constants;
mod shader;
mod transaction;
mod util;

use crate::{
    push_constants::PushConstants,
    util::{check_k_nibbles, hash_to_iv, sha256},
};

const K: u32 = 8;

// TODO: while waiting on the gpu invocation, bruteforce on the cpu.
fn main() {
    let genesis_hash: [u32; 8];
    let genesis_transaction = transaction::Transaction::default();
    let padded = util::pad(genesis_transaction.to_string().as_bytes());
    let mut words = util::to_words(&padded);
    let nonce_index = genesis_transaction.nonce_offset();
    let mut context = context::Context::new(&words);
    let mut push_constants = PushConstants {
        generation: 0,
        word_count: words.len() as u32,
        nonce_index: nonce_index,
        words: context.words_address().into(),
        nonce: context.nonce_address().into(),
        k: K,
    };
    let mut hash_count = 0;
    // genesis transaction
    {
        let mut nonce = 0;
        while nonce == 0 {
            nonce = context.invoke(&push_constants);
            hash_count += 8192 * 64;
            push_constants.generation += 1;
        }
        let nonce_high = (nonce >> 32) as u32;
        let nonce_low = (nonce & 0xFFFFFFFF) as u32;
        let idx = nonce_index as usize;
        words[idx] = nonce_high;
        words[idx + 1] = nonce_low;
        let hash_u32 = sha256(&words);
        let result: Vec<u8> = hash_u32.iter().flat_map(|&w| w.to_be_bytes()).collect();
        genesis_hash = hash_to_iv(&result);
        if check_k_nibbles(&result, K) {
            println!("success! (genesis)");
        } else {
            println!("failure... (genesis)");
        }
    }

    let hash_2: [u32; 8];
    let transaction_2 = transaction::Transaction::new(genesis_transaction, genesis_hash);
    // 2. transaction
    {
        let padded = util::pad(transaction_2.to_string().as_bytes());
        let mut words = util::to_words(&padded);
        context.update_words(&words);
        let nonce_index = transaction_2.nonce_offset();
        let mut push_constants = PushConstants {
            generation: 0,
            word_count: words.len() as u32,
            nonce_index: nonce_index,
            words: context.words_address().into(),
            nonce: context.nonce_address().into(),
            k: K,
        };

        let mut nonce = 0;
        while nonce == 0 {
            nonce = context.invoke(&push_constants);
            hash_count += 8192 * 64;
            push_constants.generation += 1;
        }
        let nonce_high = (nonce >> 32) as u32;
        let nonce_low = (nonce & 0xFFFFFFFF) as u32;
        let idx = nonce_index as usize;
        words[idx] = nonce_high;
        words[idx + 1] = nonce_low;
        let hash_u32 = sha256(&words);
        let result: Vec<u8> = hash_u32.iter().flat_map(|&w| w.to_be_bytes()).collect();
        hash_2 = hash_to_iv(&result);
        if check_k_nibbles(&result, K) {
            println!("success! (2.)");
        } else {
            println!("failure... (2.)");
        }
    }
    // 3. transaction
    {
        let transaction_3 = transaction::Transaction::new(transaction_2.clone(), hash_2);
        let padded = util::pad(transaction_3.to_string().as_bytes());
        let mut words = util::to_words(&padded);
        context.update_words(&words);
        let nonce_index = transaction_3.nonce_offset();
        let mut push_constants = PushConstants {
            generation: 0,
            word_count: words.len() as u32,
            nonce_index: nonce_index,
            words: context.words_address().into(),
            nonce: context.nonce_address().into(),
            k: K,
        };

        let mut nonce = 0;
        while nonce == 0 {
            nonce = context.invoke(&push_constants);
            hash_count += 8192 * 64;
            push_constants.generation += 1;
        }
        let nonce_high = (nonce >> 32) as u32;
        let nonce_low = (nonce & 0xFFFFFFFF) as u32;
        let idx = nonce_index as usize;
        words[idx] = nonce_high;
        words[idx + 1] = nonce_low;
        let hash_u32 = sha256(&words);
        let result: Vec<u8> = hash_u32.iter().flat_map(|&w| w.to_be_bytes()).collect();
        if check_k_nibbles(&result, K) {
            println!("success! (3.)");
        } else {
            println!("failure... (3.)");
        }
    }
    println!("total hashes: {hash_count}");
}
