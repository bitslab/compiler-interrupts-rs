# compiler-interrupts

[![crates.io](https://img.shields.io/crates/v/compiler-interrupts.svg)](https://crates.io/crates/compiler-interrupts)
[![docs.rs](https://docs.rs/compiler-interrupts/badge.svg)](https://docs.rs/compiler-interrupts)
[![license](https://img.shields.io/crates/l/compiler-interrupts.svg)](LICENSE)

`compiler-interrupts` provides Rust API for the Compiler Interrupts library. Check out the Compiler Interrupts [main repository](https://github.com/bitslab/CompilerInterrupts) for more info.

## Requirements

* [Rust 1.45.0](https://www.rust-lang.org/tools/install) or later is required. Due to the usage of [`#[thread_local]`](https://github.com/rust-lang/rust/issues/29594) unstable feature, this package currently requires nightly Rust.

## Getting started

Add this to your `Cargo.toml`.

``` toml
[dependencies]
compiler-interrupts = "1.0"
```

Register the Compiler Interrupts handler in your program.

``` rust
fn interrupt_handler(ic: i64) {
    println!("Compiler interrupt called with instruction count: {}", ic);
}

fn main() {
    unsafe {
        compiler_interrupts::register(1000, 1000, interrupt_handler);
    }

    // your code
    for _ in 0..100 {
        println!("hello world!");
    }
}
```

If you have [`cargo-compiler-interrupts`](https://github.com/bitslab/cargo-compiler-interrupts) installed, you can now run `cargo build-ci` to start the compilation and integration processes. Check out the **[documentation](https://docs.rs/compiler-interrupts)** for more info about the API.

## Contribution

All issue reports, feature requests, pull requests and GitHub stars are welcomed and much appreciated.

## Author

Quan Tran ([@quanshousio](https://quanshousio.com))

## Acknowledgements

* My advisor [Jakob Eriksson](https://www.linkedin.com/in/erikssonjakob) for the enormous support for this project.
* [Nilanjana Basu](https://www.linkedin.com/in/nilanjana-basu-99027959) for implementing the Compiler Interrupts.

## License

`cargo-compiler-interrupts` is available under the MIT license. See the [LICENSE](LICENSE) file for more info.
