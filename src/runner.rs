use perf_event::events::{Cache, CacheOp, CacheResult, Hardware, Software, WhichCache};
use perf_event::{Builder, Group};
use thousands::Separable;

/*
#[repr(u32)]
pub enum Hardware {
    CPU_CYCLES,
    INSTRUCTIONS,
    CACHE_REFERENCES,
    CACHE_MISSES,
    BRANCH_INSTRUCTIONS,
    BRANCH_MISSES,
    BUS_CYCLES,
    STALLED_CYCLES_FRONTEND,
    STALLED_CYCLES_BACKEND,
    REF_CPU_CYCLES,
}

#[repr(u32)]
pub enum Software {
    CPU_CLOCK,
    TASK_CLOCK,
    PAGE_FAULTS,
    CONTEXT_SWITCHES,
    CPU_MIGRATIONS,
    PAGE_FAULTS_MIN,
    PAGE_FAULTS_MAJ,
    ALIGNMENT_FAULTS,
    EMULATION_FAULTS,
    DUMMY,
}

pub struct Cache {
    pub which: WhichCache,
    pub operation: CacheOp,
    pub result: CacheResult,
}

#[repr(u32)]
pub enum WhichCache {
    L1D,
    L1I,
    LL,
    DTLB,
    ITLB,
    BPU,
    NODE,
}

#[repr(u32)]
pub enum CacheOp {
    READ,
    WRITE,
    PREFETCH,
}

#[repr(u32)]
pub enum CacheResult {
    ACCESS,
    MISS,
}
*/

pub fn run_benchmarks(name: &str, callback: impl Fn(), iterations: usize) -> std::io::Result<()> {
    let skip_all_this = false;
    if skip_all_this {
        callback();
        return Ok(());
    }

    // A `Group` lets us enable and disable several counters atomically.
    let mut group = Group::new()?;

    let task_clock = Builder::new()
        .group(&mut group)
        .kind(Software::TASK_CLOCK)
        .build()?;
    let context_switches = Builder::new()
        .group(&mut group)
        .kind(Software::CONTEXT_SWITCHES)
        .build()?;
    let cpu_migrations = Builder::new()
        .group(&mut group)
        .kind(Software::CPU_MIGRATIONS)
        .build()?;
    let page_faults = Builder::new()
        .group(&mut group)
        .kind(Software::PAGE_FAULTS)
        .build()?;

    let cycles = Builder::new()
        .group(&mut group)
        .kind(Hardware::CPU_CYCLES)
        .build()?;
    let instructions = Builder::new()
        .group(&mut group)
        .kind(Hardware::INSTRUCTIONS)
        .build()?;

    let cache_accesses = Builder::new()
        .group(&mut group)
        .kind(Hardware::CACHE_REFERENCES)
        .build()?;
    let l1_cache_loads = Builder::new()
        .group(&mut group)
        .kind(Cache {
            which: WhichCache::L1D,
            operation: CacheOp::READ,
            result: CacheResult::ACCESS,
        })
        .build()?;
    let l1_cache_misses = Builder::new()
        .group(&mut group)
        .kind(Cache {
            which: WhichCache::L1D,
            operation: CacheOp::READ,
            result: CacheResult::MISS,
        })
        .build()?;
    let l1_cache_prefetches = Builder::new()
        .group(&mut group)
        .kind(Cache {
            which: WhichCache::L1D,
            operation: CacheOp::PREFETCH,
            result: CacheResult::ACCESS,
        })
        .build()?;

    // We need to separate L2 cache events into their own group,
    // because they're incompatible with some of the events of
    // the first group.
    let mut group_2 = Group::new()?;

    let l2_cache_accesses_from_dc_misses = Builder::new()
        .group(&mut group_2)
        .raw_config(0xc860)
        .build()?;
    let l2_cache_hits_from_dc_misses = Builder::new()
        .group(&mut group_2)
        .raw_config(0x7064)
        .build()?;

    group.enable()?;
    callback();
    group.disable()?;

    group_2.enable()?;
    callback();
    group_2.disable()?;

    /*
    We want to display something like this:

    ====================================================================
            234.04 msec task-clock                       #
                 1      context-switches                 #    4.273 /sec
                 0      cpu-migrations                   #    0.000 /sec
                72      page-faults                      #  307.634 /sec
       916,694,940      cycles                           #    3.917 GHz
     3,768,251,802      instructions                     #    4.11   per cycle

     1,009,884,042      L1-dcache-loads
            25,093      L1-dcache-load-misses            #    0.00% of all L1-dcache accesses
            12,925      L1-dcache-prefetches
            25,098      l2_cache_accesses_from_dc_misses
            13,680      l2_cache_hits_from_dc_misses
    ====================================================================
    */

    let counts = group.read()?;
    let counts_2 = group_2.read()?;

    let task_clock_nsec = counts[&task_clock] as f64;
    let task_clock_msec = counts[&task_clock] as f64 / 1_000_000.0;
    let task_clock_s = counts[&task_clock] as f64 / 1_000_000.0;

    println!("====================================================================");
    println!("Benchmarking {}... ", name);

    println!(
        "{count:>16.2} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = task_clock_msec,
        unit = "msec",
        name = "task-clock",
        info = "",
        info_unit = "",
    );
    println!(
        "{count:>16.2} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts[&context_switches].separate_with_underscores(),
        unit = "",
        name = "context-switches",
        info = counts[&context_switches] as f64 / task_clock_s,
        info_unit = "/sec",
    );
    println!(
        "{count:>16.2} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts[&cpu_migrations].separate_with_underscores(),
        unit = "",
        name = "cpu-migrations",
        info = counts[&cpu_migrations] as f64 / task_clock_s,
        info_unit = "/sec",
    );
    println!(
        "{count:>16.2} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts[&page_faults].separate_with_underscores(),
        unit = "",
        name = "page-faults",
        info = counts[&page_faults] as f64 / task_clock_s,
        info_unit = "/sec",
    );
    println!("");

    println!(
        "{count:>16} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts[&cycles].separate_with_underscores(),
        unit = "",
        name = "cycles",
        info = counts[&cycles] as f64 / task_clock_nsec,
        info_unit = "GHz",
    );
    println!(
        "{count:>16} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts[&instructions].separate_with_underscores(),
        unit = "",
        name = "instructions",
        info = counts[&instructions] as f64 / counts[&cycles] as f64,
        info_unit = "per cycle",
    );
    println!("");

    println!(
        "{count:>16} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts[&cache_accesses].separate_with_underscores(),
        unit = "",
        name = "cache accesses",
        info = "",
        info_unit = "",
    );
    println!(
        "{count:>16} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts[&l1_cache_loads].separate_with_underscores(),
        unit = "",
        name = "L1D cache loads",
        info = "",
        info_unit = "",
    );
    println!(
        "{count:>16} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts[&l1_cache_misses].separate_with_underscores(),
        unit = "",
        name = "L1D cache misses",
        info = (counts[&l1_cache_misses] as f64 / counts[&l1_cache_loads] as f64) * 100.0,
        info_unit = "% of L1D accesses",
    );
    println!(
        "{count:>16} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts[&l1_cache_prefetches].separate_with_underscores(),
        unit = "",
        name = "L1D cache prefetches",
        info = "",
        info_unit = "",
    );

    println!(
        "{count:>16} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts_2[&l2_cache_accesses_from_dc_misses].separate_with_underscores(),
        unit = "",
        name = "L2 accesses from L1 misses",
        info = "",
        info_unit = "",
    );
    println!(
        "{count:>16} {unit:<4} {name:<30} # {info:.3} {info_unit}",
        count = counts_2[&l2_cache_hits_from_dc_misses].separate_with_underscores(),
        unit = "",
        name = "L2 hits from L1 misses",
        info = (counts_2[&l2_cache_hits_from_dc_misses] as f64
            / counts_2[&l2_cache_accesses_from_dc_misses] as f64)
            * 100.0,
        info_unit = "% of L2 accesses",
    );

    Ok(())
}

/*

> perf stat -e cycles,L1-dcache-loads,L1-dcache-load-misses,L1-dcache-prefetches,l2_cache_accesses_from_dc_misses,l2_cache_hits_from_dc_misses ./target/release/test-prefetch

perf_event_open({
    type=PERF_TYPE_HARDWARE,
    size=PERF_ATTR_SIZE_VER7,
    config=PERF_COUNT_HW_CPU_CYCLES,
    sample_period=0,
    sample_type=PERF_SAMPLE_IDENTIFIER,
    read_format=
        PERF_FORMAT_TOTAL_TIME_ENABLED
        |PERF_FORMAT_TOTAL_TIME_RUNNING,
    disabled=1,
    inherit=1,
    enable_on_exec=1,
    precise_ip=0 /* arbitrary skid */,
    exclude_guest=1,
...}, 9112, -1, -1, PERF_FLAG_FD_CLOEXEC) = 3

perf_event_open({
    type=PERF_TYPE_HW_CACHE,
    size=PERF_ATTR_SIZE_VER7,
    config=
        PERF_COUNT_HW_CACHE_RESULT_ACCESS<<16
        |PERF_COUNT_HW_CACHE_OP_READ<<8
        |PERF_COUNT_HW_CACHE_L1D,
    sample_period=0,
    sample_type=PERF_SAMPLE_IDENTIFIER,
    read_format=
        PERF_FORMAT_TOTAL_TIME_ENABLED
        |PERF_FORMAT_TOTAL_TIME_RUNNING,
    disabled=1,
    inherit=1,
    enable_on_exec=1,
    precise_ip=0 /* arbitrary skid */,
    exclude_guest=1,
...}, 9112, -1, -1, PERF_FLAG_FD_CLOEXEC) = 4

perf_event_open({
    type=PERF_TYPE_HW_CACHE,
    size=PERF_ATTR_SIZE_VER7,
    config=
        PERF_COUNT_HW_CACHE_RESULT_MISS<<16
        |PERF_COUNT_HW_CACHE_OP_READ<<8
        |PERF_COUNT_HW_CACHE_L1D,
    sample_period=0,
    sample_type=PERF_SAMPLE_IDENTIFIER,
    read_format=
        PERF_FORMAT_TOTAL_TIME_ENABLED
        |PERF_FORMAT_TOTAL_TIME_RUNNING,
    disabled=1,
    inherit=1,
    enable_on_exec=1,
    precise_ip=0 /* arbitrary skid */,
    exclude_guest=1,
...}, 9112, -1, -1, PERF_FLAG_FD_CLOEXEC) = 5

perf_event_open({
    type=PERF_TYPE_HW_CACHE,
    size=PERF_ATTR_SIZE_VER7,
    config=
        PERF_COUNT_HW_CACHE_RESULT_ACCESS<<16
        |PERF_COUNT_HW_CACHE_OP_PREFETCH<<8
        |PERF_COUNT_HW_CACHE_L1D,
    sample_period=0,
    sample_type=PERF_SAMPLE_IDENTIFIER,
    read_format=
        PERF_FORMAT_TOTAL_TIME_ENABLED
        |PERF_FORMAT_TOTAL_TIME_RUNNING,
    disabled=1,
    inherit=1,
    enable_on_exec=1,
    precise_ip=0 /* arbitrary skid */,
    exclude_guest=1,
...}, 9112, -1, -1, PERF_FLAG_FD_CLOEXEC) = 7

perf_event_open({
    type=PERF_TYPE_RAW,
    size=PERF_ATTR_SIZE_VER7,
    config=0xc860,
    sample_period=0,
    sample_type=PERF_SAMPLE_IDENTIFIER,
    read_format=
        PERF_FORMAT_TOTAL_TIME_ENABLED
        |PERF_FORMAT_TOTAL_TIME_RUNNING,
    disabled=1,
    inherit=1,
    enable_on_exec=1,
    precise_ip=0 /* arbitrary skid */,
    exclude_guest=1,
...}, 9112, -1, -1, PERF_FLAG_FD_CLOEXEC) = 8

perf_event_open({
    type=PERF_TYPE_RAW,
    size=PERF_ATTR_SIZE_VER7,
    config=0x7064,
    sample_period=0,
    sample_type=PERF_SAMPLE_IDENTIFIER,
    read_format=
        PERF_FORMAT_TOTAL_TIME_ENABLED
        |PERF_FORMAT_TOTAL_TIME_RUNNING,
    disabled=1,
    inherit=1,
    enable_on_exec=1,
    precise_ip=0 /* arbitrary skid */,
    exclude_guest=1,
...}, 9112, -1, -1, PERF_FLAG_FD_CLOEXEC) = 9

---

- In code

perf_event_open({
    type=PERF_TYPE_SOFTWARE,
    size=PERF_ATTR_SIZE_VER7,
    config=PERF_COUNT_SW_DUMMY,
    sample_period=0,
    sample_type=0,
    read_format=
        PERF_FORMAT_TOTAL_TIME_ENABLED
        |PERF_FORMAT_TOTAL_TIME_RUNNING
        |PERF_FORMAT_ID
        |PERF_FORMAT_GROUP,
    disabled=1,
    exclude_kernel=1,
    exclude_hv=1,
    precise_ip=0 /* arbitrary skid */,
...}, 0, -1, -1, 0) = 3

perf_event_open({
    type=PERF_TYPE_HW_CACHE,
    size=PERF_ATTR_SIZE_VER7,
    config=
        PERF_COUNT_HW_CACHE_RESULT_ACCESS<<16
        |PERF_COUNT_HW_CACHE_OP_READ<<8
        |PERF_COUNT_HW_CACHE_L1D,
    sample_period=0,
    sample_type=0,
    read_format=
        PERF_FORMAT_TOTAL_TIME_ENABLED
        |PERF_FORMAT_TOTAL_TIME_RUNNING,
    exclude_kernel=1,
    exclude_hv=1,
    precise_ip=0 /* arbitrary skid */,
...}, 0, -1, 3, 0) = 4

perf_event_open({
    type=PERF_TYPE_HW_CACHE,
    size=PERF_ATTR_SIZE_VER7,
    config=
        PERF_COUNT_HW_CACHE_RESULT_ACCESS<<16
        |PERF_COUNT_HW_CACHE_OP_READ<<8
        |PERF_COUNT_HW_CACHE_LL,
    sample_period=0,
    sample_type=0,
    read_format=
        PERF_FORMAT_TOTAL_TIME_ENABLED
        |PERF_FORMAT_TOTAL_TIME_RUNNING,
    exclude_kernel=1,
    exclude_hv=1,
    precise_ip=0 /* arbitrary skid */,
...}, 0, -1, 3, 0) = -1 ENOENT (No such file or directory)


































*/
