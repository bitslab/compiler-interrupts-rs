//! [![crates.io](https://img.shields.io/crates/v/compiler-interrupts.svg)][crates.io]
//! [![docs.rs](https://docs.rs/compiler-interrupts/badge.svg)][docs.rs]
//! [![license](https://img.shields.io/crates/l/compiler-interrupts.svg)][license]
//!
//! `compiler-interrupts` provides Rust API for the Compiler Interrupts library.
//! Check out the Compiler Interrupts [main repository][compiler-interrupts] for more info.
//!
//! ## Requirements
//!
//! * [Rust 1.45.0][rust] or later is required.
//! Due to the usage of [`#[thread_local]`][thread_local] unstable feature,
//! this package currently requires nightly Rust.
//!
//! ## Getting started
//!
//! Add this to your `Cargo.toml`.
//!
//! ``` toml
//! [dependencies]
//! compiler-interrupts = "1.0"
//! ```
//!
//! Register the Compiler Interrupts handler in your program.
//!
//! ``` rust
//! #![feature(thread_local)]
//!
//! #[thread_local]
//! #[allow(non_upper_case_globals)]
//! static mut prev_ic: i64 = 0;
//!
//! fn interrupt_handler(ic: i64) {
//!     let interval;
//!     unsafe {
//!         // save the last interval
//!         interval = ic - prev_ic;
//!
//!         // update the instruction count
//!         prev_ic = ic;
//!     }
//!     if interval < 0 {
//!         panic!("IR interval was negative")
//!     }
//!     println!(
//!         "CI @ {}: last interval = {} IR",
//!         std::thread::current().name().expect("invalid thread name"),
//!         interval
//!     );
//! }
//!
//! fn main() {
//!     // register the CI handler for 1000 IR and cycles interval
//!     unsafe {
//!         compiler_interrupts::register(1000, 1000, interrupt_handler);
//!     }
//!
//!     // do something compute-intensive
//!     for _ in 0..100 {}
//!
//!     println!("i can add an unsafe block, right?")
//! }
//! ```
//!
//! If you have [`cargo-compiler-interrupts`][cargo-compiler-interrupts] installed,
//! you can now run `cargo build-ci` to start the compilation and integration.
//! Check out the **[documentation][docs.rs]** for more info about the API.
//!
//! ## Contribution
//!
//! All issue reports, feature requests, pull requests and GitHub stars are welcomed
//! and much appreciated.
//!
//! ## Author
//!
//! Quan Tran ([@quanshousio][quanshousio])
//!
//! ## Acknowledgements
//!
//! * My advisor [Jakob Eriksson][jakob] for the enormous support for this project.
//! * [Nilanjana Basu][nilanjana] for implementing the Compiler Interrupts.
//!
//! ## License
//!
//! `compiler-interrupts` is available under the MIT license.
//! See the [LICENSE][license] file for more info.
//!
//! [crates.io]: https://crates.io/crates/compiler-interrupts
//! [docs.rs]: https://docs.rs/compiler-interrupts
//! [license]: https://github.com/bitslab/compiler-interrupts-rs/blob/main/LICENSE
//! [compiler-interrupts]: https://github.com/bitslab/CompilerInterrupts
//! [rust]: https://www.rust-lang.org/tools/install
//! [thread_local]: https://github.com/rust-lang/rust/issues/29594
//! [cargo-compiler-interrupts]: https://github.com/bitslab/cargo-compiler-interrupts
//! [quanshousio]: https://quanshousio.com
//! [jakob]: https://www.linkedin.com/in/erikssonjakob
//! [nilanjana]: https://www.linkedin.com/in/nilanjana-basu-99027959

#![feature(thread_local)]

/// Default large interval
const LARGE_INTERVAL: i64 = 100000;

/// Default small interval
const SMALL_INTERVAL: i64 = 10000;

/// Interrupt function for the framework.
#[no_mangle]
#[thread_local]
static mut intvActionHook: fn(i64) = dummy;

/// Store the interrupt handler from [`register`].
#[allow(non_upper_case_globals)]
#[thread_local]
static mut int_handler: fn(i64) = dummy;

/// Store the enable hook from [`register_enable_hook`].
#[allow(non_upper_case_globals)]
#[thread_local]
static mut enableHook: Option<fn()> = None;

/// Store the disable hook from [`register_disable_hook`].
#[allow(non_upper_case_globals)]
#[thread_local]
static mut disableHook: Option<fn()> = None;

/// IR interrupt interval for the framework.
#[no_mangle]
#[thread_local]
static mut ci_ir_interval: i64 = LARGE_INTERVAL;

/// IR interrupt reset interval when target target cycles is not exceeded
/// for the framework.
#[no_mangle]
#[thread_local]
static mut ci_reset_ir_interval: i64 = LARGE_INTERVAL / 2;

/// Cycles interrupt interval for the framework.
#[no_mangle]
#[thread_local]
static mut ci_cycles_interval: i64 = SMALL_INTERVAL;

/// Cycles interrupt threshold to fire the interrupt or reset the IR counter
/// for the framework.
#[no_mangle]
#[thread_local]
static mut ci_cycles_threshold: i64 = (0.9 * LARGE_INTERVAL as f64) as i64;

/// Thread-local local counter for the framework.
#[no_mangle]
#[thread_local]
static mut LocalLC: i32 = 0;

/// Thread-local disable counter for the framework.
#[no_mangle]
#[thread_local]
static mut lc_disabled_count: i32 = 0;

/// Thread-local next interval for the framework.
#[no_mangle]
#[thread_local]
static mut NextInterval: i32 = 0;

/// A dummy function.
fn dummy(_: i64) {}

/// Assigns the interrupt function to itself and calls the handler from [`register`].
fn interrupt_handler(ic: i64) {
    unsafe {
        intvActionHook = dummy;
        int_handler(ic);
        intvActionHook = interrupt_handler
    }
}

/// Registers a handler for Compiler Interrupts.
///
/// This function takes a IR interval, cycles interval, and
/// function pointer to the Compiler Interrupts handler.
/// The handler receives an approximation of the number of IR instructions
/// since the last interrupt as the argument.
///
/// # Note
///
/// This function is thread-specific, which means it only registers
/// on the thread they called on.
///
/// This function should not be called multiple times.
/// Consecutive calls will override the previous intervals and handler.
///
/// # Safety
///
/// This function mutates a thread-local static variable which uses for the interrupt handler.
/// Thread unsafety will not be introduced. However, calling the handler outside Rust would
/// probably violate Rust's safe memory model; hence the function is considered unsafe.
///
/// # Examples
///
/// ```
/// fn interrupt_handler(ic: i64) {
///     println!("Compiler interrupt called with instruction count: {}", ic);
/// }
///
/// unsafe {
///     compiler_interrupts::register(10000, 10000, interrupt_handler);
/// }
/// ```
pub unsafe fn register(ir_interval: i64, cycles_interval: i64, handler: fn(i64)) {
    LocalLC += ci_ir_interval as i32;
    ci_ir_interval = ir_interval;
    ci_reset_ir_interval = ir_interval / 2;
    ci_cycles_interval = cycles_interval;
    ci_cycles_threshold = (0.9 * cycles_interval as f64) as i64;
    int_handler = handler;
    intvActionHook = interrupt_handler;
}

/// De-registers the handler for Compiler Interrupts.
///
/// This function removes the given interrupts handler from [`register`].
///
/// # Note
///
/// This function is thread-specific, which means it only de-registers
/// on the thread they called on.
///
/// This function should not be called multiple times.
/// Consecutive calls will do nothing as the handler has already been de-registered.
///
/// # Safety
///
/// This function mutates a thread-local static variable which uses for the interrupt handler.
/// Thread unsafety will not be introduced. Rust considers mutating static variable unsafe.
pub unsafe fn deregister() {
    ci_ir_interval = LARGE_INTERVAL;
    ci_reset_ir_interval = LARGE_INTERVAL / 2;
    ci_cycles_interval = LARGE_INTERVAL;
    ci_cycles_threshold = (0.9 * LARGE_INTERVAL as f64) as i64;
    int_handler = dummy;
    intvActionHook = dummy;
}

/// Enables Compiler Interrupts.
///
/// This function enables Compiler Interrupts if it is currently disabled.
/// Compiler Interrupts are enabled by default.
///
/// # Note
///
/// This function is thread-specific, which means it only enables
/// on the thread they called on.
///
/// This function can be called multiple times.
/// Number of [`enable`] calls must be the same as the number of previous [`disable`] calls
/// to re-enable the interrupts.
///
/// # Safety
///
/// This function mutates a thread-local static variable which uses for the counter.
/// Thread unsafety will not be introduced. Rust considers mutating static variable unsafe.
///
/// # Examples
///
/// ```
/// unsafe {
///     // disables the interrupts
///     compiler_interrupts::disable();
/// }
///
/// for _ in 0..42 {
///     println!("interrupts have been disabled");
/// }
///
/// unsafe {
///     // re-enables the interrupts
///     compiler_interrupts::enable();
/// }
/// ```
pub unsafe fn enable() {
    if lc_disabled_count > 0 {
        lc_disabled_count -= 1;
    }
    if let Some(hook) = enableHook {
        hook();
    }
    if lc_disabled_count == 0 {
        intvActionHook = interrupt_handler;
    }
}

/// Disables Compiler Interrupts.
///
/// This function disables Compiler Interrupts if it is currently enabled.
///
/// # Note
///
/// This function is thread-specific, which means it only disables
/// on the thread they called on.
///
/// This function can be called multiple times.
/// Consecutive calls will do nothing as the interrupts are already disabled.
///
/// # Safety
///
/// This function mutates a thread-local static variable which uses for the counter.
/// Thread unsafety will not be introduced. Rust considers mutating static variable unsafe.
///
/// # Examples
///
/// ```
/// unsafe {
///     // disables the interrupts
///     compiler_interrupts::disable();
/// }
///
/// for _ in 0..42 {
///     println!("interrupts have been disabled");
/// }
///
/// unsafe {
///     // re-enables the interrupts
///     compiler_interrupts::enable();
/// }
/// ```
pub unsafe fn disable() {
    intvActionHook = dummy;
    lc_disabled_count += 1;
    if let Some(hook) = disableHook {
        hook();
    }
}

/// Registers a hook when enabling Compiler Interrupts.
///
/// This function takes a function pointer to be called after enabling Compiler Interrupts.
/// Compiler Interrupts can be enabled by calling [`enable`].
///
/// # Note
///
/// This function is thread-specific, which means it only registers
/// on the thread they called on.
///
/// This function should not be called multiple times.
/// Consecutive calls will override the previous hook.
///
/// # Safety
///
/// This function mutates a thread-local static variable which uses for the hook.
/// Thread unsafety will not be introduced. Rust considers mutating static variable unsafe.
pub unsafe fn register_enable_hook(hook: fn()) {
    enableHook = Some(hook)
}

/// De-registers the hook when enabling Compiler Interrupts.
///
/// This function removes the given hook from [`register_enable_hook`].
///
/// # Note
///
/// This function is thread-specific, which means it only de-registers
/// on the thread they called on.
///
/// This function should not be called multiple times.
/// Consecutive calls will do nothing as the hook has already been removed.
///
/// # Safety
///
/// This function mutates a thread-local static variable which uses for the hook.
/// Thread unsafety will not be introduced. Rust considers mutating static variable unsafe.
pub unsafe fn deregister_enable_hook() {
    enableHook = None
}

/// Registers a hook when disabling Compiler Interrupts.
///
/// This function takes a function pointer to be called before disabling Compiler Interrupts.
/// Compiler Interrupts can be temporarily disabled by calling [`disable`].
///
/// # Note
///
/// This function is thread-specific, which means it only registers
/// on the thread they called on.
///
/// This function should not be called multiple times.
/// Consecutive calls will override the previous hook.
///
/// # Safety
///
/// This function mutates a thread-local static variable which uses for the hook.
/// Thread unsafety will not be introduced. Rust considers mutating static variable unsafe.
pub unsafe fn register_disable_hook(hook: fn()) {
    disableHook = Some(hook)
}

/// De-registers the hook when disabling Compiler Interrupts.
///
/// This function removes the given hook from [`register_disable_hook`].
///
/// # Note
///
/// This function is thread-specific, which means it only de-registers
/// on the thread they called on.
///
/// This function should not be called multiple times.
/// Consecutive calls will do nothing as the hook has already been removed.
///
/// # Safety
///
/// This function mutates a thread-local static variable which uses for the hook.
/// Thread unsafety will not be introduced. Rust considers mutating static variable unsafe.
pub unsafe fn deregister_disable_hook() {
    disableHook = None
}

/// Enables the probe instrumentation.
///
/// # Note
///
/// This function is thread-specific, which means it only enables
/// on the thread they called on.
///
/// # Safety
///
/// This function is called outside the normal Rust program.
#[no_mangle]
pub unsafe fn instr_enable() {}

/// Disables the probe instrumentation.
///
/// # Note
///
/// This function is thread-specific, which means it only disables
/// on the thread they called on.
///
/// # Safety
///
/// This function is called outside the normal Rust program.
#[no_mangle]
pub unsafe fn instr_disable() {}
