mod context;
mod transaction;
mod words;

// TODO: implement sha256 block creation
// TODO: create DEVICE_LOCAL buffer for SHA256 blocks
// TODO: create HOST_VISIBLE buffer for nonce
// TODO: initialize Push Constants (and increase with each iteration)
// TODO: read nonce buffer
// TODO: check if nonce is valid and validate again on the CPU (add sha2 as a dependency for sha256)
// TODO: implement nonce_buffer_index calculation (for a u64 nonce)

fn main() {
    let transaction = transaction::Transaction::default();
    println!("{transaction}");
    let _context = context::Context::new();
}
