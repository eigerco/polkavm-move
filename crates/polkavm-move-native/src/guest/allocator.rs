use polkavm_derive::LeakingAllocator;

#[global_allocator]
static GLOBAL: LeakingAllocator = LeakingAllocator;
