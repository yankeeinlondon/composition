---
name: rayon
description: Expert knowledge for data parallelism in Rust using Rayon - parallel iterators, fork-join primitives, work-stealing scheduler, thread pools, and CPU-bound parallel processing. Use when parallelizing computations, processing large collections, implementing divide-and-conquer algorithms, or optimizing CPU-intensive Rust code.
last_updated: 2025-12-19T20:30:00Z
hash: b40eb47a1f1f0715
---

# Rayon

Rayon is the gold standard for **data parallelism** in Rust. While `tokio` is designed for I/O-bound concurrency (waiting on network/disk), Rayon is built for CPU-bound parallelism - making your math, data processing, and recursive algorithms utilize every core with minimal code changes.

## Core Principles

- **Use `par_iter()` for collection processing** - Convert sequential iterators by changing `.iter()` to `.par_iter()`
- **Use `join()` for fork-join patterns** - Split work into two parallel branches when both are needed
- **Use `scope()` for borrowed data** - Spawn tasks that borrow from the stack with lifetime guarantees
- **Avoid parallelizing tiny workloads** - Parallelism has overhead; small tasks run slower in parallel
- **Never block Rayon threads** - Blocking on mutexes/channels inside Rayon tasks causes deadlocks
- **Use custom pools for isolation** - Create separate thread pools to limit resources or isolate workloads
- **Expect non-deterministic ordering** - Floating-point reductions may vary between runs

## Quick Reference

```rust
use rayon::prelude::*;

// Parallel iterator - process collection across all cores
let results: Vec<_> = data.par_iter().map(|x| expensive(x)).collect();

// Parallel mutable iteration
data.par_iter_mut().for_each(|x| *x = transform(*x));

// Parallel sort (much faster for large datasets)
data.par_sort();
data.par_sort_by_key(|x| x.field);

// Fork-join - run two tasks in parallel
use rayon::join;
let (a, b) = join(|| compute_left(), || compute_right());

// Scoped parallelism - borrow from stack
rayon::scope(|s| {
    s.spawn(|_| task_one(&borrowed_data));
    s.spawn(|_| task_two(&borrowed_data));
});
```

## Work-Stealing Scheduler

Rayon maintains a fixed-size thread pool (matching CPU core count by default). Each thread has a deque of tasks. When a thread finishes its work, it steals tasks from other threads' queues - keeping CPU utilization high even with uneven task durations.

## Common Patterns

### Parallel Data Processing

```rust
use rayon::prelude::*;

let squares: Vec<i32> = input
    .par_iter()
    .map(|&i| i * i)
    .collect();
```

### Recursive Divide-and-Conquer

```rust
use rayon::join;

fn quick_sort<T: PartialOrd + Send>(v: &mut [T]) {
    if v.len() <= 1 { return; }
    let mid = partition(v);
    let (low, high) = v.split_at_mut(mid);
    join(|| quick_sort(low), || quick_sort(high));
}
```

### Custom Thread Pool

```rust
let pool = rayon::ThreadPoolBuilder::new()
    .num_threads(4)
    .build()
    .unwrap();

pool.install(|| {
    // All par_iter() calls here use this pool's 4 threads
    data.par_iter().for_each(|x| process(x));
});
```

## Gotchas and Fixes

| Problem | Cause | Fix |
|---------|-------|-----|
| Parallel slower than sequential | Workload too small per task | Use `.with_min_len(n)` to increase chunk size |
| Deadlock | Blocking on mutex/channel inside Rayon | Use `join`/`scope` for dependencies, not blocking primitives |
| Non-reproducible float results | Non-deterministic reduction order | Use sequential iterators for bit-exact float math |

### Controlling Granularity

```rust
// Force minimum 1000 elements per chunk to reduce overhead
data.par_iter()
    .with_min_len(1000)
    .map(|x| small_work(x))
    .collect()
```

## Debugging and Tuning

| Tool/Method | Purpose |
|-------------|---------|
| `RAYON_NUM_THREADS=N` | Test scaling with different core counts |
| `rayon-logs` crate | Generate timeline visualizations of task execution |
| `cargo flamegraph` | Identify if splitting overhead exceeds work time |

## When to Use Rayon vs Tokio

| Use Case | Choose |
|----------|--------|
| CPU-bound computation | **Rayon** |
| I/O-bound (network, disk) | Tokio |
| Processing large collections | **Rayon** |
| Async web servers | Tokio |
| Recursive algorithms | **Rayon** |
| Database queries | Tokio |

## Resources

- [Rayon on crates.io](https://crates.io/crates/rayon)
- [Rayon GitHub](https://github.com/rayon-rs/rayon)
- [API Documentation](https://docs.rs/rayon)
