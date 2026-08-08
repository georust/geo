//! Driver for the profiling scenarios in `geo_benches::scenarios`.
//!
//! Runs one scenario in a steady loop so a sampling profiler (samply,
//! Instruments) can attach to a single long-running process. Setup happens
//! before the loop, so samples are dominated by the algorithm under test.
//!
//! Usage:
//!   profiling --list
//!   profiling <scenario> [--seconds S | --iters N]
//!
//! Defaults to 15 seconds of steady-state work.

use std::alloc::{GlobalAlloc, Layout, System};
use std::process::exit;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use geo_benches::scenarios;

/// Wraps the system allocator with allocation and byte counters so each run
/// can report allocations per iteration alongside wall-clock time. Two relaxed
/// atomic increments per allocation; negligible next to the workloads here.
struct CountingAlloc;

static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

enum Budget {
    Seconds(f64),
    Iters(u64),
}

fn usage() -> ! {
    eprintln!("usage: profiling <scenario> [--seconds S | --iters N]");
    eprintln!("       profiling --list");
    exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let all = scenarios::all();

    if args.iter().any(|a| a == "--list") {
        let width = all.iter().map(|s| s.name.len()).max().unwrap_or(0);
        for s in &all {
            println!("{:width$}  {}", s.name, s.description);
        }
        return;
    }

    let mut name: Option<&str> = None;
    let mut budget = Budget::Seconds(15.0);
    let mut iter_args = args.iter();
    while let Some(arg) = iter_args.next() {
        match arg.as_str() {
            "--seconds" => {
                let v = iter_args.next().unwrap_or_else(|| usage());
                budget = Budget::Seconds(v.parse().unwrap_or_else(|_| usage()));
            }
            "--iters" => {
                let v = iter_args.next().unwrap_or_else(|| usage());
                budget = Budget::Iters(v.parse().unwrap_or_else(|_| usage()));
            }
            other if !other.starts_with("--") && name.is_none() => name = Some(other),
            _ => usage(),
        }
    }
    let Some(name) = name else { usage() };

    let Some(scenario) = all.iter().find(|s| s.name == name) else {
        eprintln!("unknown scenario '{name}'; run with --list to see available scenarios");
        exit(1)
    };

    eprintln!("preparing '{name}'...");
    let setup_start = Instant::now();
    let mut run = (scenario.prepare)();
    eprintln!("setup took {:.2?}; running", setup_start.elapsed());

    let start = Instant::now();
    let allocs_before = ALLOC_COUNT.load(Ordering::Relaxed);
    let bytes_before = ALLOC_BYTES.load(Ordering::Relaxed);
    let mut iterations = 0u64;
    match budget {
        Budget::Seconds(s) => {
            let limit = Duration::from_secs_f64(s);
            while start.elapsed() < limit {
                run();
                iterations += 1;
            }
        }
        Budget::Iters(n) => {
            for _ in 0..n {
                run();
                iterations += 1;
            }
        }
    }
    let elapsed = start.elapsed();
    let allocs = ALLOC_COUNT.load(Ordering::Relaxed) - allocs_before;
    let bytes = ALLOC_BYTES.load(Ordering::Relaxed) - bytes_before;
    let iters = iterations.max(1);
    eprintln!(
        "{name}: {iterations} iterations in {:.2?} ({:.2?}/iter)",
        elapsed,
        elapsed / iters as u32
    );
    eprintln!(
        "{name}: {} allocations/iter, {:.1} MiB allocated/iter",
        allocs / iters,
        (bytes / iters) as f64 / (1024.0 * 1024.0)
    );
}
