// TODO: implement a parallel bruteforcing function using rayon. it should be asynchronous and yield on a successful nonce.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use rayon::prelude::*;
use tokio_util::sync::CancellationToken;

use crate::util::{check_k_nibbles, sha256};

pub async fn bruteforce(
    words: Vec<u32>,
    k: u32,
    nonce_index: usize,
    token: CancellationToken,
    nonce_counter: Arc<AtomicU64>,
) -> Option<u64> {
    tokio::task::spawn_blocking(move || {
        let start_nonce = nonce_counter.load(Ordering::Relaxed);
        let end_nonce = start_nonce + 5_000_000;
        let result = (0..5_000_000).into_par_iter().find_map_any(|nonce| {
            if token.is_cancelled() {
                return Some(0);
            } else {
                let mut local_words = words.clone();
                let nonce_high = ((start_nonce + nonce) >> 32) as u32;
                let nonce_low = ((start_nonce + nonce) & 0xFFFFFFFF) as u32;
                local_words[nonce_index] = nonce_high;
                local_words[nonce_index + 1] = nonce_low;
                let hash = sha256(&local_words);
                let result: Vec<u8> = hash.iter().flat_map(|&w| w.to_be_bytes()).collect();

                if check_k_nibbles(&result, k) {
                    Some(nonce)
                } else {
                    None
                }
            }
        });
        match result {
            Some(r) => r,
            None => {
                nonce_counter.store(end_nonce, Ordering::Relaxed);
                0
            }
        }
    })
    .await
    .ok()
}
