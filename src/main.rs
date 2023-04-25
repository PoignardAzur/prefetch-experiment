#![allow(unused)]

mod runner;

use std::arch::asm;
use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
use std::hint::black_box;

use rand::Rng;
use runner::run_benchmarks;

macro_rules! asm_comment {
    ($tt:tt) => {
        "// {}"
    };
}

macro_rules! side_effect_read {
    ($( $x:expr ),*) => {
        unsafe {
            asm!(
                $(asm_comment!($x),)*
                $(in(reg) $x,)*
            );
        }
    };
}

macro_rules! side_effect_rw {
    ($( $x:expr ),*) => {
        unsafe {
            asm!(
                $(asm_comment!($x),)*
                $(inout(reg) $x,)*
            );
        }
    };
}

#[inline(never)]
pub fn cpp_bench<const N: usize>(array: &[u32; N]) -> u64 {
    /*
    uint32_t data[4096];
    uint32_t top = 0, bottom = 0;
    for (size_t i = 0; i < len; i += 2) {
        uint32_t elem;

        elem = data[i];
        top    += elem >> 16;
        bottom += elem & 0xFFFF;

        elem = data[i + 1];
        top    += elem >> 16;
        bottom += elem & 0xFFFF;
    }
    */

    let mut i: u32 = 0;
    let mut top: u32 = 0;
    let mut bottom: u32 = 0;

    while i < N as u32 {
        let elem = array[i as usize];
        top += elem >> 16;
        bottom += elem & 0xFFFF;

        let elem = array[(i + 1) as usize];
        top += elem >> 16;
        bottom += elem & 0xFFFF;

        i += 2;

        unsafe { asm!("") }
    }

    top as u64 + bottom as u64
}

#[inline(never)]
pub fn bench_noops<const N: usize>(_array: &[u8; N]) -> u64 {
    let x = black_box(3);

    let mut sum = black_box(0);
    for _ in 0..N {
        unsafe {
            asm!("nop", "nop", "nop", "nop", "nop", "nop");
        }
    }

    sum
}

#[inline(never)]
pub fn bench_alu_ops<const N: usize>(_array: &[u8; N]) -> u64 {
    let mut sum = (0);
    for _ in 0..N {
        sum += 3;
        side_effect_read!(sum);
    }

    sum
}

#[inline(never)]
pub fn bench_alu_ops_unrolled<const N: usize>(_array: &[u8; N]) -> u64 {
    let x = black_box(3);

    let mut sum_1 = 0;
    let mut sum_2 = 0;
    let mut sum_3 = 0;
    let mut sum_4 = 0;

    for _ in 0..N {
        sum_1 += x;
        sum_2 += x;
        sum_3 += x;
        sum_4 += x;
        side_effect_rw!(sum_1, sum_2, sum_3, sum_4);
    }

    sum_1 + sum_2 + sum_3 + sum_4
}

#[inline(never)]
pub fn bench_alu_ops_super_unrolled<const N: usize>(_array: &[u8; N]) -> u64 {
    let x = black_box(3);

    let mut sum_1 = 0;
    let mut sum_2 = 0;
    let mut sum_3 = 0;
    let mut sum_4 = 0;
    let mut sum_5 = 0;
    let mut sum_6 = 0;
    let mut sum_7 = 0;
    let mut sum_8 = 0;

    for _ in 0..N {
        unsafe {
            asm!(
                "add {sum_1}, {x}",
                "add {sum_2}, {x}",
                "add {sum_3}, {x}",
                "add {sum_4}, {x}",
                "add {sum_5}, {x}",
                "add {sum_6}, {x}",
                "add {sum_7}, {x}",
                "add {sum_8}, {x}",
                sum_1 = inout(reg) sum_1,
                sum_2 = inout(reg) sum_2,
                sum_3 = inout(reg) sum_3,
                sum_4 = inout(reg) sum_4,
                sum_5 = inout(reg) sum_5,
                sum_6 = inout(reg) sum_6,
                sum_7 = inout(reg) sum_7,
                sum_8 = inout(reg) sum_8,
                x = in(reg) x as u64,
            );
        }
    }

    sum_1 + sum_2 + sum_3 + sum_4 + sum_5 + sum_6 + sum_7 + sum_8
}

#[inline(never)]
pub fn bench_mul_ops<const N: usize>(_array: &[u8; N]) -> u64 {
    let x = black_box(3);

    let mut product = black_box(0);
    for _ in 0..N {
        product *= x;
    }

    product
}

#[inline(never)]
pub fn bench_sum_of_array<const N: usize>(array: &[u8; N]) -> u8 {
    let x = black_box(3);
    let mut sum = 0;

    for i in 0..N {
        sum += array[i] & x;
    }

    sum
}

#[inline(never)]
pub fn bench_sum_of_array_unrolled<const N: usize>(array: &[u8; N]) -> u8 {
    let x = black_box(3);
    let mut sum_1 = 0;
    let mut sum_2 = 0;

    let mut i = 0;
    while i < N {
        sum_1 += array[i] & x;
        sum_2 += array[i + 1] & x;
        i += 2;
    }

    sum_1 + sum_2
}

#[inline(never)]
pub fn bench_sum_of_array_with_stride<const N: usize>(array: &[u8; N], stride: usize) -> u8 {
    let x = black_box(3);
    let mut sum = 0;

    let mut i = 0;
    while i < N {
        sum += array[i] & x;
        i += stride;
    }

    sum
}

#[inline(never)]
pub fn bench_sum_of_array_with_stride_prefetch<const N: usize, const P: usize>(
    array: &[u8; N],
    stride: usize,
) -> u8 {
    let x = black_box(3);
    let mut sum = 0;

    let mut i = 0;
    while i < N {
        sum += array[i] & x;
        unsafe {
            _mm_prefetch(array.as_ptr().add(i + P * stride) as *const i8, _MM_HINT_T0);
        };
        i += stride;
    }

    sum
}

#[inline(never)]
pub fn bench_sum_array_stride_and_pad<const N: usize>(array: &[u8; N], stride: usize) -> u8 {
    let x = black_box(3);
    let mut sum: u64 = 0;

    let mut i = 0;
    while i < N {
        sum += array[i] as u64;
        unsafe {
            asm!(
                "xor {sum}, {sum}",
                "add {sum}, {x}",
                "add {sum}, {x}",
                "add {i}, {stride}",
                sum = inout(reg) sum,
                i = inout(reg) i,
                x = in(reg) x as u64,
                stride = in(reg) stride,
            );
        }
    }

    sum as u8
}

#[inline(never)]
pub fn bench_sum_array_changing_stride<const N: usize>(array: &[u8; N]) -> u8 {
    let x = black_box(3);
    let mut sum: u64 = 0;
    let mut stride: u64 = 0;

    let mut i = 0;
    while i < N {
        sum += array[i] as u64;
        unsafe {
            asm!(
                "xor {sum}, {sum}",
                "add {stride}, 9",
                "and {stride}, 31",
                "add {i}, 110",
                "add {i}, {stride}",
                sum = inout(reg) sum,
                i = inout(reg) i,
                stride = in(reg) stride,
            );
        }
    }

    sum as u8
}

#[inline(never)]
pub fn bench_sum_array_indirect<const N: usize, const M: usize>(
    array: &[u8; N],
    indices: &[usize; M],
) -> u8 {
    let x = black_box(3);
    let mut sum: u64 = 0;
    let mut stride: u64 = 0;

    let mut i = 0;
    while i < M {
        // TODO - check that indices are in bounds
        sum += array[indices[i]] as u64;
        i += 64;
    }

    sum as u8
}

// ----------------

pub fn main() -> std::io::Result<()> {
    let selected = black_box(4);

    const ITER_COUNT: usize = 10_000;
    let small_array = black_box([0; 1000]);

    run_benchmarks(
        "bench_noops",
        || {
            for _ in 0..ITER_COUNT {
                black_box(bench_noops(&small_array));
            }
        },
        small_array.len() * ITER_COUNT,
        None,
    )?;

    run_benchmarks(
        "bench_alu_ops",
        || {
            for _ in 0..ITER_COUNT {
                black_box(bench_alu_ops(&small_array));
            }
        },
        small_array.len() * ITER_COUNT,
        None,
    )?;

    run_benchmarks(
        "bench_alu_ops_unrolled",
        || {
            for _ in 0..ITER_COUNT {
                black_box(bench_alu_ops_unrolled(&small_array));
            }
        },
        small_array.len() * ITER_COUNT,
        None,
    )?;

    run_benchmarks(
        "bench_alu_ops_super_unrolled",
        || {
            for _ in 0..ITER_COUNT {
                black_box(bench_alu_ops_super_unrolled(&small_array));
            }
        },
        small_array.len() * ITER_COUNT,
        None,
    )?;

    run_benchmarks(
        "bench_mul_ops",
        || {
            for _ in 0..ITER_COUNT {
                black_box(bench_mul_ops(&small_array));
            }
        },
        small_array.len() * ITER_COUNT,
        None,
    )?;

    run_benchmarks(
        "bench_sum_of_array",
        || {
            for _ in 0..ITER_COUNT {
                black_box(bench_sum_of_array(&small_array));
            }
        },
        small_array.len() * ITER_COUNT,
        Some(small_array.len() * ITER_COUNT),
    )?;

    run_benchmarks(
        "bench_sum_of_array_unrolled",
        || {
            for _ in 0..ITER_COUNT {
                black_box(bench_sum_of_array_unrolled(&small_array));
            }
        },
        small_array.len() * ITER_COUNT / 2,
        Some(small_array.len() * ITER_COUNT),
    )?;

    const SMALL_ITER_COUNT: usize = 1_000;
    let array_1_mb = black_box([0; 1_000_000]);

    run_benchmarks(
        "bench_sum_array_1MB",
        || {
            for _ in 0..SMALL_ITER_COUNT {
                black_box(bench_sum_of_array_with_stride(&array_1_mb, 1));
            }
        },
        array_1_mb.len() * SMALL_ITER_COUNT,
        Some(array_1_mb.len() * ITER_COUNT),
    )?;

    run_benchmarks(
        "bench_sum_array_1MB_stride_64",
        || {
            for _ in 0..SMALL_ITER_COUNT {
                black_box(bench_sum_of_array_with_stride(&array_1_mb, 64));
            }
        },
        array_1_mb.len() * SMALL_ITER_COUNT / 64,
        Some(array_1_mb.len() * ITER_COUNT / 64),
    )?;

    run_benchmarks(
        "bench_sum_array_1MB_stride_16",
        || {
            for _ in 0..SMALL_ITER_COUNT {
                black_box(bench_sum_of_array_with_stride(&array_1_mb, 16));
            }
        },
        array_1_mb.len() * SMALL_ITER_COUNT / 16,
        Some(array_1_mb.len() * SMALL_ITER_COUNT / 16),
    )?;

    run_benchmarks(
        "bench_sum_array_1MB_stride_16_prefetch_4",
        || {
            for _ in 0..SMALL_ITER_COUNT {
                black_box(bench_sum_of_array_with_stride_prefetch::<1_000_000, 4>(
                    &array_1_mb,
                    16,
                ));
            }
        },
        array_1_mb.len() * SMALL_ITER_COUNT / 16,
        Some(array_1_mb.len() * SMALL_ITER_COUNT / 16),
    )?;

    run_benchmarks(
        "bench_sum_array_1MB_stride_16_prefetch_1",
        || {
            for _ in 0..SMALL_ITER_COUNT {
                black_box(bench_sum_of_array_with_stride_prefetch::<1_000_000, 1>(
                    &array_1_mb,
                    16,
                ));
            }
        },
        array_1_mb.len() * SMALL_ITER_COUNT / 16,
        Some(array_1_mb.len() * SMALL_ITER_COUNT / 16),
    )?;

    run_benchmarks(
        "bench_sum_array_stride_16_and_pad",
        || {
            for _ in 0..SMALL_ITER_COUNT {
                black_box(bench_sum_array_stride_and_pad(&array_1_mb, 16));
            }
        },
        array_1_mb.len() * SMALL_ITER_COUNT / 16,
        Some(array_1_mb.len() * SMALL_ITER_COUNT / 16),
    )?;

    run_benchmarks(
        "bench_sum_array_stride_128_and_pad",
        || {
            for _ in 0..SMALL_ITER_COUNT {
                black_box(bench_sum_array_stride_and_pad(&array_1_mb, 128));
            }
        },
        array_1_mb.len() * SMALL_ITER_COUNT / 128,
        Some(array_1_mb.len() * SMALL_ITER_COUNT / 128),
    )?;

    run_benchmarks(
        "bench_sum_array_changing_stride",
        || {
            for _ in 0..SMALL_ITER_COUNT {
                black_box(bench_sum_array_changing_stride(&array_1_mb));
            }
        },
        array_1_mb.len() * SMALL_ITER_COUNT / 128,
        Some(array_1_mb.len() * SMALL_ITER_COUNT / 128),
    )?;

    // generate random indices
    let array_indices: [usize; 100_000] = (0..100_000)
        .map(|_| rand::thread_rng().gen_range(0..array_1_mb.len()))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    run_benchmarks(
        "bench_sum_array_indirect",
        || {
            for _ in 0..SMALL_ITER_COUNT {
                black_box(bench_sum_array_indirect(&array_1_mb, &array_indices));
            }
        },
        array_indices.len() * SMALL_ITER_COUNT / 64,
        Some(array_indices.len() * SMALL_ITER_COUNT / 64),
    )?;

    Ok(())
}
