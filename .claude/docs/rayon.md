---
name: rayon
description: Comprehensive guide to Rayon - Rust's data parallelism library for CPU-bound workloads
created: 2025-12-19
last_updated: 2025-12-19T10:30:00Z
hash: 2535205971c29b58
tags:
  - rust
  - parallelism
  - concurrency
  - performance
  - multithreading
---

# Rayon

Rayon is the gold standard for **data parallelism** in Rust. While async runtimes like Tokio are designed for I/O-bound concurrency (waiting on network/disk), Rayon is built for CPU-bound parallelism - making your math, data processing, and recursive algorithms utilize every core on your machine with minimal code changes.

The library's philosophy is "fearless concurrency made easy" - converting sequential code to parallel often requires changing just a single method call.

## Table of Contents

- [Core Concepts](#core-concepts)
- [Parallel Iterators](#parallel-iterators)
- [Fork-Join Primitives](#fork-join-primitives)
- [Scopes](#scopes)
- [Thread Pool Configuration](#thread-pool-configuration)
- [Common Use Cases](#common-use-cases)
- [Performance Optimization](#performance-optimization)
- [Common Pitfalls](#common-pitfalls)
- [Debugging and Profiling](#debugging-and-profiling)
- [Quick Reference](#quick-reference)
- [Resources](#resources)

## Core Concepts

### Work-Stealing Scheduler

Rayon doesn't spawn a thread for every task. It maintains a fixed-size thread pool (usually matching your CPU core count). Each thread has its own deque (double-ended queue) of tasks.

When a thread finishes its work, it doesn't sit idle - it "steals" a task from the back of another thread's queue. This keeps CPU utilization high even when tasks vary significantly in duration.

```
Thread 1: [Task A] [Task B] [Task C]  <- pushes new tasks here
                                  ^
Thread 2: [empty]                 |
           steals from here ------+
```

This design provides several benefits:

- **Load balancing** - Work automatically redistributes across threads
- **Cache efficiency** - Threads prefer their own tasks, maintaining cache locality
- **Low overhead** - No central queue contention or task distribution logic

### When to Use Rayon vs Async

| Workload Type | Best Choice | Why |
|--------------|-------------|-----|
| CPU-intensive computation | Rayon | Maximizes CPU utilization |
| Network I/O | Tokio/async | Efficient waiting without blocking threads |
| File I/O (many files) | Tokio/async | Handles concurrent I/O efficiently |
| Data transformation | Rayon | Parallel processing of collections |
| Mixed CPU + I/O | Both | Use Rayon for CPU work, async for I/O |

## Parallel Iterators

The most famous Rayon feature. Converting a standard iterator to a parallel one is often as simple as changing `.iter()` to `.par_iter()`.

### Basic Usage

```rust
use rayon::prelude::*;

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Sequential
    let squares_seq: Vec<i32> = numbers.iter().map(|&x| x * x).collect();

    // Parallel - just change iter() to par_iter()
    let squares_par: Vec<i32> = numbers.par_iter().map(|&x| x * x).collect();
}
```

### Iterator Types

| Method | Ownership | Use Case |
|--------|-----------|----------|
| `par_iter()` | Borrows `&T` | Read-only access |
| `par_iter_mut()` | Borrows `&mut T` | In-place modification |
| `into_par_iter()` | Takes ownership `T` | Consuming the collection |

### Common Parallel Operations

```rust
use rayon::prelude::*;

let data: Vec<i32> = (0..1000).collect();

// Parallel map
let doubled: Vec<i32> = data.par_iter().map(|x| x * 2).collect();

// Parallel filter
let evens: Vec<i32> = data.par_iter().filter(|x| *x % 2 == 0).cloned().collect();

// Parallel reduce
let sum: i32 = data.par_iter().sum();

// Parallel fold (with identity and combining function)
let sum: i32 = data.par_iter()
    .fold(|| 0, |acc, &x| acc + x)
    .sum();

// Parallel find
let first_big: Option<&i32> = data.par_iter().find_any(|&&x| x > 500);

// Parallel for_each (side effects)
data.par_iter().for_each(|x| {
    // Process each element
    println!("{}", x);
});

// Parallel sort
let mut sortable = data.clone();
sortable.par_sort();

// Parallel sort with custom comparator
sortable.par_sort_by(|a, b| b.cmp(a)); // Descending
```

### Chaining Operations

Parallel iterators can be chained just like regular iterators:

```rust
use rayon::prelude::*;

let result: i32 = (0..1_000_000)
    .into_par_iter()
    .filter(|x| x % 2 == 0)
    .map(|x| x * x)
    .filter(|x| x % 3 == 0)
    .sum();
```

## Fork-Join Primitives

For explicit parallel task execution, Rayon provides `join` and `scope`.

### The `join` Function

Runs two closures in parallel and waits for both to complete:

```rust
use rayon::join;

fn main() {
    let (result_a, result_b) = join(
        || expensive_computation_a(),
        || expensive_computation_b(),
    );

    println!("A: {}, B: {}", result_a, result_b);
}
```

### Recursive Divide-and-Conquer

`join` excels at recursive algorithms like QuickSort or MergeSort:

```rust
use rayon::join;

fn parallel_quicksort<T: PartialOrd + Send>(v: &mut [T]) {
    if v.len() <= 1 {
        return;
    }

    let mid = partition(v);
    let (low, high) = v.split_at_mut(mid);

    // Run both halves in parallel
    join(
        || parallel_quicksort(low),
        || parallel_quicksort(&mut high[1..]), // Skip pivot
    );
}

fn partition<T: PartialOrd>(v: &mut [T]) -> usize {
    let pivot = v.len() - 1;
    let mut i = 0;
    for j in 0..pivot {
        if v[j] <= v[pivot] {
            v.swap(i, j);
            i += 1;
        }
    }
    v.swap(i, pivot);
    i
}
```

### Parallel Tree Traversal

```rust
use rayon::join;

struct TreeNode {
    value: i32,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

fn parallel_sum(node: &TreeNode) -> i32 {
    let (left_sum, right_sum) = join(
        || node.left.as_ref().map_or(0, |n| parallel_sum(n)),
        || node.right.as_ref().map_or(0, |n| parallel_sum(n)),
    );

    node.value + left_sum + right_sum
}
```

## Scopes

The `scope` API allows spawning tasks that can borrow data from the surrounding stack:

```rust
use rayon::scope;

fn main() {
    let mut results = vec![0; 4];
    let data = vec![1, 2, 3, 4];

    scope(|s| {
        for (result, &input) in results.iter_mut().zip(&data) {
            s.spawn(move |_| {
                *result = expensive_computation(input);
            });
        }
    });
    // All spawned tasks complete before scope returns

    println!("{:?}", results);
}
```

### Nested Scopes

Scopes can be nested for complex dependency patterns:

```rust
use rayon::scope;

scope(|outer| {
    outer.spawn(|inner| {
        // Task A
        inner.spawn(|_| { /* Task A.1 */ });
        inner.spawn(|_| { /* Task A.2 */ });
    });
    outer.spawn(|_| {
        // Task B (runs in parallel with A)
    });
});
```

## Thread Pool Configuration

### Global Pool

By default, Rayon uses a global thread pool sized to your CPU core count. You can customize it at startup:

```rust
use rayon::ThreadPoolBuilder;

fn main() {
    // Configure before any parallel work
    ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .expect("Failed to initialize thread pool");

    // Now all par_iter() calls use 4 threads
}
```

### Custom Pools

Create isolated pools for specific workloads:

```rust
use rayon::ThreadPoolBuilder;

fn main() {
    let cpu_pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .thread_name(|i| format!("cpu-worker-{}", i))
        .build()
        .unwrap();

    let io_pool = ThreadPoolBuilder::new()
        .num_threads(8)
        .thread_name(|i| format!("io-worker-{}", i))
        .build()
        .unwrap();

    // Run work in specific pool
    cpu_pool.install(|| {
        // All par_iter() calls here use cpu_pool's 4 threads
        (0..1000).into_par_iter().for_each(|x| {
            compute_intensive_work(x);
        });
    });
}
```

### Pool Configuration Options

```rust
ThreadPoolBuilder::new()
    .num_threads(8)                          // Thread count
    .thread_name(|i| format!("worker-{}", i)) // Thread naming
    .stack_size(4 * 1024 * 1024)             // Stack size per thread
    .start_handler(|i| println!("Thread {} starting", i))
    .exit_handler(|i| println!("Thread {} exiting", i))
    .panic_handler(|panic| eprintln!("Thread panicked: {:?}", panic))
    .build()
    .unwrap();
```

## Common Use Cases

### Image Processing

```rust
use rayon::prelude::*;

struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

fn apply_grayscale(image: &mut [Pixel]) {
    image.par_iter_mut().for_each(|pixel| {
        let gray = ((pixel.r as u32 + pixel.g as u32 + pixel.b as u32) / 3) as u8;
        pixel.r = gray;
        pixel.g = gray;
        pixel.b = gray;
    });
}
```

### Matrix Operations

```rust
use rayon::prelude::*;

fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rows = a.len();
    let cols = b[0].len();
    let inner = b.len();

    (0..rows)
        .into_par_iter()
        .map(|i| {
            (0..cols)
                .map(|j| {
                    (0..inner).map(|k| a[i][k] * b[k][j]).sum()
                })
                .collect()
        })
        .collect()
}
```

### Batch File Processing

```rust
use rayon::prelude::*;
use std::path::PathBuf;

fn process_files(paths: &[PathBuf]) -> Vec<ProcessedResult> {
    paths
        .par_iter()
        .filter_map(|path| {
            match process_single_file(path) {
                Ok(result) => Some(result),
                Err(e) => {
                    eprintln!("Error processing {:?}: {}", path, e);
                    None
                }
            }
        })
        .collect()
}
```

### Monte Carlo Simulation

```rust
use rayon::prelude::*;
use rand::Rng;

fn estimate_pi(samples: usize) -> f64 {
    let inside: usize = (0..samples)
        .into_par_iter()
        .map(|_| {
            let mut rng = rand::thread_rng();
            let x: f64 = rng.gen();
            let y: f64 = rng.gen();
            if x * x + y * y <= 1.0 { 1 } else { 0 }
        })
        .sum();

    4.0 * inside as f64 / samples as f64
}
```

## Performance Optimization

### Chunk Size Control

Control how work is divided among threads:

```rust
use rayon::prelude::*;

let data: Vec<i32> = (0..1_000_000).collect();

// Force minimum chunk size
let result: Vec<i32> = data
    .par_iter()
    .with_min_len(1000)  // Each thread gets at least 1000 elements
    .map(|x| x * 2)
    .collect();

// Force maximum chunk size
let result: Vec<i32> = data
    .par_iter()
    .with_max_len(10000) // Split into chunks of at most 10000
    .map(|x| x * 2)
    .collect();
```

### When Parallelism Helps

Rayon benefits workloads where:

- **Work per element is significant** - The computation dominates overhead
- **Collection is large** - Enough work to distribute
- **No dependencies between elements** - Elements can be processed independently

```rust
// Good candidate - expensive per-element work
let results: Vec<_> = large_items.par_iter()
    .map(|item| expensive_crypto_hash(item))
    .collect();

// Poor candidate - trivial per-element work
let results: Vec<_> = numbers.par_iter()
    .map(|x| x + 1)  // Too cheap, overhead dominates
    .collect();
```

### Benchmarking Parallel vs Sequential

Always benchmark to verify speedup:

```rust
use std::time::Instant;
use rayon::prelude::*;

fn benchmark() {
    let data: Vec<i32> = (0..10_000_000).collect();

    // Sequential
    let start = Instant::now();
    let _: Vec<i32> = data.iter().map(|x| x * x).collect();
    println!("Sequential: {:?}", start.elapsed());

    // Parallel
    let start = Instant::now();
    let _: Vec<i32> = data.par_iter().map(|x| x * x).collect();
    println!("Parallel: {:?}", start.elapsed());
}
```

## Common Pitfalls

### Pitfall 1: Tiny Workloads

**Problem:** Parallelism has overhead. If the work inside your closure is trivial (like adding 1 to an integer), `par_iter()` will be **slower** than a sequential loop.

```rust
// BAD - overhead exceeds benefit
let result: Vec<i32> = small_vec.par_iter().map(|x| x + 1).collect();

// GOOD - use sequential for trivial work
let result: Vec<i32> = small_vec.iter().map(|x| x + 1).collect();
```

**Fix:** Use `with_min_len()` to ensure minimum chunk sizes, or don't parallelize trivial operations:

```rust
// If you must parallelize, set minimum chunk size
let result: Vec<i32> = data
    .par_iter()
    .with_min_len(10000)
    .map(|x| x + 1)
    .collect();
```

### Pitfall 2: Blocking Inside Rayon Tasks

**Problem:** Rayon threads are a finite resource. Using `Mutex`, channels, or any blocking operation inside a Rayon task that waits for another Rayon task can exhaust the thread pool and cause deadlocks.

```rust
// DANGEROUS - can deadlock
use std::sync::Mutex;

let results = Mutex::new(Vec::new());
(0..1000).into_par_iter().for_each(|x| {
    let computed = expensive_work(x);
    results.lock().unwrap().push(computed); // Contention!
});
```

**Fix:** Use Rayon's own primitives (`collect`, `reduce`, `fold`) instead of external synchronization:

```rust
// GOOD - let Rayon handle collection
let results: Vec<_> = (0..1000)
    .into_par_iter()
    .map(|x| expensive_work(x))
    .collect();
```

### Pitfall 3: Floating-Point Non-Determinism

**Problem:** Rayon processes elements in non-deterministic order. Operations that aren't perfectly associative (like floating-point addition) may yield slightly different results between runs due to rounding errors.

```rust
// Results may vary slightly between runs
let sum: f64 = floats.par_iter().sum();
```

**Fix:** If bit-for-bit reproducibility is required, use sequential iterators for floating-point reductions:

```rust
// Deterministic
let sum: f64 = floats.iter().sum();
```

### Pitfall 4: Forgetting `Send` Bounds

**Problem:** Data shared across threads must implement `Send`. Non-`Send` types (like `Rc`) cause compilation errors.

```rust
// WON'T COMPILE - Rc is not Send
use std::rc::Rc;
let data: Vec<Rc<i32>> = vec![Rc::new(1), Rc::new(2)];
let _: Vec<_> = data.par_iter().map(|x| **x * 2).collect();
```

**Fix:** Use `Arc` for shared ownership in parallel contexts:

```rust
use std::sync::Arc;
let data: Vec<Arc<i32>> = vec![Arc::new(1), Arc::new(2)];
let _: Vec<_> = data.par_iter().map(|x| **x * 2).collect();
```

## Debugging and Profiling

### Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `RAYON_NUM_THREADS` | Override thread count | `RAYON_NUM_THREADS=4 cargo run` |

### Scaling Analysis

Test how your code scales with different core counts:

```bash
# Test with 1, 2, 4, 8 threads
for n in 1 2 4 8; do
    echo "Threads: $n"
    RAYON_NUM_THREADS=$n cargo run --release
done
```

### Profiling Tools

| Tool | Purpose |
|------|---------|
| `cargo flamegraph` | Identify if overhead dominates actual work |
| `rayon-logs` crate | Generate timeline visualizations of task execution |
| `perf` (Linux) | Low-level CPU profiling |

### Using rayon-logs

```rust
// In Cargo.toml
// [dependencies]
// rayon-logs = "0.1"

use rayon_logs::ThreadPoolBuilder;

fn main() {
    let pool = ThreadPoolBuilder::new()
        .build()
        .expect("Failed to build pool");

    let (_, log) = pool.logging_install(|| {
        // Your parallel code here
    });

    log.save_svg("timeline.svg").expect("Failed to save");
}
```

## Quick Reference

### Essential Imports

```rust
use rayon::prelude::*;  // Parallel iterators
use rayon::join;        // Fork-join
use rayon::scope;       // Scoped tasks
use rayon::ThreadPoolBuilder; // Pool configuration
```

### Iterator Conversion Cheat Sheet

| Sequential | Parallel |
|------------|----------|
| `.iter()` | `.par_iter()` |
| `.iter_mut()` | `.par_iter_mut()` |
| `.into_iter()` | `.into_par_iter()` |
| `for x in collection` | `collection.par_iter().for_each(\|x\| ...)` |

### Common Methods

| Method | Description |
|--------|-------------|
| `par_iter()` | Parallel iterator over references |
| `par_iter_mut()` | Parallel iterator over mutable references |
| `into_par_iter()` | Parallel consuming iterator |
| `par_sort()` | Parallel sort (stable) |
| `par_sort_unstable()` | Parallel sort (faster, not stable) |
| `join(a, b)` | Run two tasks in parallel |
| `scope(\|s\| s.spawn(...))` | Scoped parallel tasks |

### Performance Guidelines

| Collection Size | Work Per Element | Recommendation |
|----------------|------------------|----------------|
| < 1,000 | Any | Sequential (unless work is expensive) |
| 1,000 - 100,000 | Trivial | Sequential or `with_min_len()` |
| 1,000 - 100,000 | Moderate+ | Parallel |
| > 100,000 | Any | Parallel |

## Resources

- [Official Rayon Documentation](https://docs.rs/rayon/latest/rayon/)
- [Rayon GitHub Repository](https://github.com/rayon-rs/rayon)
- [Rayon Parallel Iterator Trait](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html)
- [The Rust Book - Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [rayon-logs Crate](https://crates.io/crates/rayon-logs) - Task visualization
