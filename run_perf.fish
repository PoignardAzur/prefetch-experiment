#! /usr/bin/fish

perf stat -e task-clock,context-switches,cpu-migrations,page-faults,cycles,instructions,l2_cache_accesses_from_dc_misses,l2_cache_hits_from_dc_misses ./target/release/test-prefetch
perf stat -e cycles,L1-dcache-loads,L1-dcache-load-misses,L1-dcache-prefetches,l2_cache_accesses_from_dc_misses,l2_cache_hits_from_dc_misses ./target/release/test-prefetch
