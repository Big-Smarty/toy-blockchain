import "sha256";

// TODO: use buffer device addresses instead of descriptors; DONE
// TODO: push the successful nonce back to the CPU; DONE
// TODO: check for successful hash on the gpu

struct PushConstants {
  uint32_t generation;
  uint32_t word_count;
  uint32_t nonce_index;
  uint32_t k;
  uint32_t *words;
  uint64_t *nonce;
};

[[vk::push_constant]]
PushConstants push_constants;

uint64_t get_nonce(uint32_t id, uint32_t generation) {
  uint3 workgroup_size = WorkgroupSize();
  uint3 workgroup_count = WorkgroupCount();
  uint64_t nonce = (generation * (workgroup_size.x * workgroup_count.x)) + id;
  return nonce;
}

bool check_k_nibbles(uint32_t[8] hash, uint32_t k) {
  uint32_t zero_nibbles = 0;
  for (uint32_t nibble_idx = 0; nibble_idx < 32; nibble_idx++) {
    uint32_t word_idx = nibble_idx / 8;
    uint32_t nibble_in_word = nibble_idx % 8;
    uint32_t nibble = (hash[word_idx] >> (28 - nibble_in_word * 4)) & 0xF;
    if (nibble == 0) {
      zero_nibbles++;
    } else {
      break;
    }
  }
  return zero_nibbles >= k;
}

[shader("compute")]
[numthreads(32, 1, 1)]
void main(uint3 ID: SV_DispatchThreadID) {
  uint32_t idx = ID.x;
  uint64_t nonce = get_nonce(idx, push_constants.generation);
  uint32_t[8] hash_result =
      sha256(push_constants.words, push_constants.word_count, nonce,
             push_constants.nonce_index);
  if (check_k_nibbles(hash_result, push_constants.k)) {
    push_constants.nonce[0] = nonce;
  }
  bool all_zeroes = true;
  for (uint32_t i = 0; i < 8; i++) {
    if (hash_result[i] != 0) {
      all_zeroes = false;
    }
  }
  if (all_zeroes) {
    push_constants.nonce[0] = 9999999999999;
  }
}
