# compiler-interrupts

[![crates.io](https://img.shields.io/crates/v/compiler-interrupts.svg)][crates.io]
[![docs.rs](https://docs.rs/compiler-interrupts/badge.svg)][docs.rs]
[![license](https://img.shields.io/crates/l/compiler-interrupts.svg)][license]

`compiler-interrupts` provides Rust API for the Compiler Interrupts library.
Check out the Compiler Interrupts [main repository][compiler-interrupts] for more info.

## Requirements

* [Rust 1.45.0][rust] or later is required.
Due to the usage of [`#[thread_local]`][thread_local] unstable feature,
this package currently requires nightly Rust.

## Getting started

Add this to your `Cargo.toml`.

``` toml
[dependencies]
compiler-interrupts = "1.0"
```

Register the Compiler Interrupts handler in your program.

``` rust
#![feature(thread_local)]

#[thread_local]
#[allow(non_upper_case_globals)]
static mut prev_ic: i64 = 0;

fn interrupt_handler(ic: i64) {
    let interval;
    unsafe {
        // save the last interval
        interval = ic - prev_ic;

        // update the instruction count
        prev_ic = ic;
    }
    if interval < 0 {
        panic!("IR interval was negative")
    }
    println!(
        "CI @ {}: last interval = {} IR",
        std::thread::current().name().expect("invalid thread name"),
        interval
    );
}

fn main() {
    // register the CI handler for 1000 IR and cycles interval
    unsafe {
        compiler_interrupts::register(1000, 1000, interrupt_handler);
    }

    // do something compute-intensive
    for _ in 0..100 {}

    println!("i can add an unsafe block, right?")
}
```

If you have [`cargo-compiler-interrupts`][cargo-compiler-interrupts] installed,
you can now run `cargo build-ci` to start the compilation and integration.
Check out the **[documentation][docs.rs]** for more info about the API.

## Contribution

All issue reports, feature requests, pull requests and GitHub stars are welcomed
and much appreciated.

## Author

Quan Tran ([@quanshousio][quanshousio])

## Acknowledgements

* My advisor [Jakob Eriksson][jakob] for the enormous support for this project.
* [Nilanjana Basu][nilanjana] for implementing the Compiler Interrupts.

## License

`compiler-interrupts` is available under the MIT license.
See the [LICENSE][license] file for more info.

[crates.io]: https://crates.io/crates/compiler-interrupts
[docs.rs]: https://docs.rs/compiler-interrupts
[license]: https://github.com/bitslab/compiler-interrupts-rs/blob/main/LICENSE
[compiler-interrupts]: https://github.com/bitslab/CompilerInterrupts
[rust]: https://www.rust-lang.org/tools/install
[thread_local]: https://github.com/rust-lang/rust/issues/29594
[cargo-compiler-interrupts]: https://github.com/bitslab/cargo-compiler-interrupts
[quanshousio]: https://quanshousio.com
[jakob]: https://www.linkedin.com/in/erikssonjakob
[nilanjana]: https://www.linkedin.com/in/nilanjana-basu-99027959
