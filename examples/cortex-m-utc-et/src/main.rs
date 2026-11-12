#![no_std]
#![no_main]

mod cases;
use core::{hint::black_box, ptr};
use cortex_m::peripheral::DWT;
use cortex_m_rt::entry;
use embedded_alloc::LlffHeap;
use panic_halt as _;
use stkcraft_time::{
    contract::{NaifEt, TimeContext},
    Calendar, LeapSecondTable,
};

#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();
const HEAP_BYTES: usize = 16 * 1024;
#[repr(align(8))]
struct HeapMemory([u8; HEAP_BYTES]);
static mut HEAP_MEMORY: HeapMemory = HeapMemory([0; HEAP_BYTES]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Stats {
    pub total_cycles: u64,
    pub min_cycles: u32,
    pub max_cycles: u32,
}
impl Stats {
    const EMPTY: Self = Self {
        total_cycles: 0,
        min_cycles: u32::MAX,
        max_cycles: 0,
    };
    fn record(&mut self, cycles: u32) {
        self.total_cycles += cycles as u64;
        self.min_cycles = self.min_cycles.min(cycles);
        self.max_cycles = self.max_cycles.max(cycles);
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Results {
    // 0=not started, 1=initializing, 2=reference checks passed,
    // 3=no active DWT counter, 4=complete. Panic is a halted incomplete run.
    pub status: u32,
    pub iterations_per_direction: u32,
    pub forward: Stats,
    pub inverse: Stats,
}
#[no_mangle]
pub static mut BENCH_RESULT: Results = Results {
    status: 0,
    iterations_per_direction: 0,
    forward: Stats::EMPTY,
    inverse: Stats::EMPTY,
};
fn publish(result: Results) {
    // The debugger reads this only after stopping execution. Firmware has one writer.
    unsafe {
        ptr::write_volatile(ptr::addr_of_mut!(BENCH_RESULT), result);
    }
}

#[entry]
fn main() -> ! {
    let mut result = Results {
        status: 1,
        iterations_per_direction: 1000,
        forward: Stats::EMPTY,
        inverse: Stats::EMPTY,
    };
    publish(result);
    // Application startup only: the static buffer is aligned, uniquely owned,
    // live forever, and the allocator is initialized once before any allocation.
    unsafe {
        HEAP.init(ptr::addr_of_mut!(HEAP_MEMORY.0) as usize, HEAP_BYTES);
    }
    let lsk = LeapSecondTable::builtin().unwrap();
    let context = TimeContext::new(&lsk);
    cases::check(context).unwrap();
    result.status = 2;
    publish(result);

    let mut peripherals = cortex_m::Peripherals::take().unwrap();
    if !DWT::has_cycle_counter() {
        result.status = 3;
        publish(result);
        halt();
    }
    peripherals.DCB.enable_trace();
    peripherals.DWT.enable_cycle_counter();
    let start = DWT::cycle_count();
    for _ in 0..128 {
        cortex_m::asm::nop();
    }
    if DWT::cycle_count().wrapping_sub(start) == 0 {
        result.status = 3;
        publish(result);
        halt();
    }

    // LSK parsing and reference checks are outside the timing region.
    // Each sample includes counter-read, black_box and Result-handling overhead.
    for i in 0..result.iterations_per_direction as usize {
        let utc = cases::CASES[i % cases::CASES.len()].0;
        let start = DWT::cycle_count();
        black_box(
            context
                .utc_to_et(&black_box(utc), Calendar::Gregorian)
                .unwrap(),
        );
        result
            .forward
            .record(DWT::cycle_count().wrapping_sub(start));
    }
    for i in 0..result.iterations_per_direction as usize {
        let et = black_box(
            NaifEt::from_seconds(f64::from_bits(cases::CASES[i % cases::CASES.len()].1)).unwrap(),
        );
        let start = DWT::cycle_count();
        black_box(
            context
                .et_to_utc(black_box(et), Calendar::Gregorian)
                .unwrap(),
        );
        result
            .inverse
            .record(DWT::cycle_count().wrapping_sub(start));
    }
    result.status = 4;
    publish(result);
    halt()
}
fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
