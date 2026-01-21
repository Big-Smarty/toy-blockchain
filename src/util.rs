// calculate manual padding for string to align the nonce such that it has its own two words
pub fn calculate_manual_pad(size: usize) -> usize {
    return size % 4;
}

pub fn _calculate_k(size: usize) -> usize {
    return (448 - 1 - size % 512 + 512) % 512;
}

pub fn pad(data: &[u8]) -> Vec<u8> {
    let mut out = data.to_vec();
    let original_bit_size = (data.len() as u64) * 8;

    out.push(0x80);

    let current_len = out.len() % 64;
    let k = if current_len <= 56 {
        56 - current_len
    } else {
        64 + 56 - current_len
    };

    for _ in 0..k {
        out.push(0x00);
    }

    out.extend_from_slice(&original_bit_size.to_be_bytes());

    out
}

pub fn to_words(data: &Vec<u8>) -> Vec<u32> {
    assert!(data.len() % 64 == 0);
    (0..data.len() / 4)
        .map(|i| {
            ((data[i * 4] as u32) << 24)
                | ((data[i * 4 + 1] as u32) << 16)
                | ((data[i * 4 + 2] as u32) << 8)
                | (data[i * 4 + 3] as u32)
        })
        .collect()
}

pub fn nonce_to_raw_string(nonce: u64) -> String {
    let mut out = String::new();
    let chars = nonce.to_be_bytes().map(|b| b as char);
    for c in chars {
        out.push(c);
    }
    out
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn bsig0(x: u32) -> u32 {
    rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22)
}

fn bsig1(x: u32) -> u32 {
    rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25)
}

fn ssig0(x: u32) -> u32 {
    rotr(x, 7) ^ rotr(x, 18) ^ (x >> 3)
}

fn ssig1(x: u32) -> u32 {
    rotr(x, 17) ^ rotr(x, 19) ^ (x >> 10)
}

pub fn sha256(words: &[u32]) -> [u32; 8] {
    let mut working_h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let chunk_count = words.len() / 16;
    for i in 0..chunk_count {
        let mut w = [0u32; 64];
        for t in 0..16 {
            w[t] = words[16 * i + t];
        }
        for t in 16..64 {
            w[t] = ssig1(w[t - 2])
                .wrapping_add(w[t - 7])
                .wrapping_add(ssig0(w[t - 15]))
                .wrapping_add(w[t - 16]);
        }
        let mut a: u32 = working_h[0];
        let mut b: u32 = working_h[1];
        let mut c: u32 = working_h[2];
        let mut d: u32 = working_h[3];
        let mut e: u32 = working_h[4];
        let mut f: u32 = working_h[5];
        let mut g: u32 = working_h[6];
        let mut h: u32 = working_h[7];
        for t in 0..64 {
            let t1 = h
                .wrapping_add(bsig1(e))
                .wrapping_add(ch(e, f, g))
                .wrapping_add(K[t])
                .wrapping_add(w[t]);
            let t2 = bsig0(a).wrapping_add(maj(a, b, c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        working_h[0] = working_h[0].wrapping_add(a);
        working_h[1] = working_h[1].wrapping_add(b);
        working_h[2] = working_h[2].wrapping_add(c);
        working_h[3] = working_h[3].wrapping_add(d);
        working_h[4] = working_h[4].wrapping_add(e);
        working_h[5] = working_h[5].wrapping_add(f);
        working_h[6] = working_h[6].wrapping_add(g);
        working_h[7] = working_h[7].wrapping_add(h);
    }
    working_h
}

pub fn check_k_nibbles(hash: &[u8], k: u32) -> bool {
    let mut zero_nibbles = 0;
    for h in hash {
        // Check high nibble
        if (h >> 4) == 0 {
            zero_nibbles += 1;
        } else {
            return zero_nibbles >= k;
        }

        if zero_nibbles >= k {
            return true;
        }

        // Check low nibble
        if (h & 0x0F) == 0 {
            zero_nibbles += 1;
        } else {
            return zero_nibbles >= k;
        }

        if zero_nibbles >= k {
            return true;
        }
    }
    return zero_nibbles >= k;
}

pub fn hash_to_iv(hash: &Vec<u8>) -> [u32; 8] {
    assert!(hash.len() == 32);

    let mut out: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..8 {
        for j in 0..4 {
            out[i] |= (hash[i * 4 + j] as u32) << (24 - (j * 8));
        }
    }

    out
}
