import "sha256";

// TODO: create a vulkan application which runs this compute shader
// NOTE: about the vulkan application:
// - MUST upload blocks
// - MUST precalculate the index of the word the nonce should be written into
// - MUST profit???

struct PushConstants {
  uint32_t generation;
  uint32_t block_count;
  uint32_t nonce_index;
};

[[vk::push_constant]]
PushConstants push_constants;

[[vk::binding(0, 0)]]
StructuredBuffer<uint32_t> blocks;

[[vk::binding(0, 1)]]
RWStructuredBuffer<uint32_t> hash;

uint32_t get_nonce(uint32_t id, uint32_t generation) {
  uint3 workgroup_size = WorkgroupSize();
  uint3 workgroup_count = WorkgroupCount();
  uint32_t nonce = (generation * (workgroup_size.x * workgroup_count.x)) + id;
  return nonce;
}

[shader("compute")]
[numthreads(32, 1, 1)]
void main(uint3 ID: SV_DispatchThreadID) {
  uint32_t idx = ID.x;
  uint32_t nonce = get_nonce(idx, push_constants.generation);
  uint32_t[8] hash_result = sha256(blocks, push_constants.block_count);
  for (uint32_t i; i < 8; i++) {
    hash[i] = hash_result[i];
  }
}
