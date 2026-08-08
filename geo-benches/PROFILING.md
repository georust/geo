# Profiling workflow

This crate contains, in addition to the criterion benchmarks, a set of
profiling scenarios: realistic workloads over large fixtures, intended to be
run under a sampling profiler in order to identify optimisation candidates.
The scenarios live in `src/scenarios.rs` and are driven by the `profiling`
binary (`src/bin/profiling.rs`).

Each scenario performs its setup (fixture parsing, input construction) once,
before the measured loop, so profiler samples are dominated by the algorithm
under test. Inputs are deterministic – fixtures are static, and randomised
inputs use a seeded RNG – so profiles are comparable across runs.

## Building

The workspace defines a `profiling` profile (release optimisation plus debug
symbols) so profilers can symbolise stacks, including inlined frames:

```sh
cargo build --profile profiling -p geo-benches --bin profiling
```

## Running

```sh
# list available scenarios
./target/profiling/profiling --list

# run one scenario for 15 seconds (the default)
./target/profiling/profiling boolean-ops-nl-zones

# or a fixed iteration count / different duration
./target/profiling/profiling relate-jts --iters 5
./target/profiling/profiling buffer-norway --seconds 30
```

## CPU profiling with samply (macOS and Linux)

```sh
cargo install samply
samply setup   # one-off, macOS only

samply record --save-only -o /tmp/boolean-ops-nl-zones.json.gz \
  ./target/profiling/profiling boolean-ops-nl-zones --seconds 15
samply load /tmp/boolean-ops-nl-zones.json.gz
```

Profiles open in the Firefox Profiler with inline stacks and source view. Do
not commit profile files to the repository.

Criterion bench binaries can also be profiled directly, since
`[profile.bench]` builds with debug symbols. Criterion's `--profile-time` flag
runs the measurement loop without saving results:

```sh
cargo bench -p geo-benches --bench relate --no-run   # prints the binary path
samply record ./target/release/deps/relate-<hash> --bench --profile-time 15
```

## Allocation profiling with Instruments (macOS)

```sh
cargo install cargo-instruments   # requires full Xcode

cargo instruments -t Allocations --profile profiling \
  -p geo-benches --bin profiling -- relate-jts --iters 3
```

Use a small iteration count: allocation recording is slow and traces grow
quickly.

## Linux instruction-level profiling

The scenario functions are deliberately plain `fn() -> Box<dyn FnMut()>`
pairs so that a [gungraun](https://github.com/gungraun/gungraun)
(valgrind/callgrind-based) bench target can wrap them for deterministic
instruction counts, cache simulation, and DHAT heap profiles on Linux, where
valgrind is available. That target is not yet present.
