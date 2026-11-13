# Minimal Cortex-M UTC/ET benchmark

This independent example calls the local stkcraft-time 0.2.1 Rust crate
with default features disabled. It does not modify the calculation kernel.
The supplied memory layout is for STM32F407VG (Cortex-M4): 1 MiB FLASH and
128 KiB main SRAM; the separate CCM region is not used. Adapt memory.x before
using another chip. The target is thumbv7em-none-eabi (software floating point).
No UART, RTT, HAL, network, RTOS or clock-tree configuration is required.

## Build and check

From this directory:

```sh
rustup target add thumbv7em-none-eabi
cargo run --release --locked --bin host-check
cargo build --release --locked --bin cortex-m-utc-et --target thumbv7em-none-eabi
```

The ELF is target/thumbv7em-none-eabi/release/cortex-m-utc-et. This builds a
complete vector table and links a firmware image, rather than merely checking
the library. The host-check uses the same three cases and the no_std library
with libm, but it executes on the host and is not an ARM execution test.

## What the firmware does

1. Initialize a dedicated 16 KiB heap. The core is no_std + alloc, not heapless.
   This is a chosen heap budget, not a measured minimum or peak-usage claim.
2. Load the built-in naif0012.tls snapshot once, outside all timed regions.
3. Verify three independent CSPICE N0067 UTC-to-ET bit patterns, then check
   three civil roundtrips. The inputs cover J2000, the 2016 leap second and
   a 2024 date. These are deterministic test queries, not mission observations.
4. Check that DWT CYCCNT exists and actually advances; otherwise publish
   status=3 and stop. Do not substitute fabricated timing on unsupported cores.
5. Measure 1,000 UTC-to-ET calls and 1,000 independent ET-to-UTC calls. Inputs
   and outputs use black_box. Each direction rotates over the three cases.
6. Publish raw cycles in BENCH_RESULT and wait. No semihosting is needed.

## Run on hardware

Use your existing SWD probe/debugger to program the ELF on an STM32F407VG.
Reset, run, then pause execution and inspect the exported BENCH_RESULT symbol
with DWARF type information:

- status=0: not started; 1: initialization/reference-check phase.
- status=2: reference checks passed; benchmark not completed.
- status=3: no usable running DWT counter; timings must not be reported.
- status=4: complete; read forward/inverse total_cycles, min_cycles, max_cycles.
- A panic halts with an incomplete status. Do not interpret partial fields
  as a successful result. Allocation failure also halts.

Mean cycles = total_cycles / iterations_per_direction. If and only if the
actual CPU clock is known and constant, seconds/call = mean_cycles / clock_Hz.
This example does not configure 168 MHz or infer a clock from the chip rating.
Timing includes counter-read, black_box and Result-handling overhead. No
counter-overhead subtraction is applied. Initialization and LSK parsing are
excluded. No interrupts are deliberately enabled by this program; record any
interrupts/debug effects in a hardware report. Per-call wrapping subtraction
assumes each operation takes less than 2^32 cycles. Test outside the debugger
if debugger timing effects matter to the intended measurement.

Record board revision, CPU/clock settings, flash wait states/cache settings,
probe/debug state, compiler, target, build flags, LSK identity, status and raw
cycle statistics. Cortex-M4 has single-precision hardware floating point;
ET remains f64 and this example deliberately uses software floating point.

## Verification boundaries

Build logs and release verification record cross-build and host checks. Static RAM includes the reserved heap but excludes maximum
stack consumption. Allocatable initialized-section bytes exclude padding and
are not a measured bootloader/firmware-update payload length. ELF debug-file
size is not flash usage. No hardware execution, maximum stack, peak heap,
cycle count, microsecond timing or flight qualification has been measured.

The three expected ET bit patterns come from tests/golden/contract_utc.csv
in STKCraft-time and were independently rechecked with SpiceyPy 8.3.0 /
CSPICE N0067 using the same LSK. Matching these samples does not prove every
epoch or kernel. On-target reference checks must pass before reporting a run.

This independent example is included in the STKCraft-time repository.
Cargo.lock pins dependencies; the core is referenced from the checkout.
For a standalone consumer, remove the dependency path and use the matching
crates.io release after it is published.

## Sources

- https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html
- https://docs.rs/embedded-alloc/0.6.0/embedded_alloc/
- https://docs.rs/cortex-m/0.7.7/cortex_m/peripheral/struct.DWT.html
- https://docs.rs/cortex-m-rt/0.7.3/cortex_m_rt/
- https://www.st.com/en/microcontrollers-microprocessors/stm32f407vg.html
