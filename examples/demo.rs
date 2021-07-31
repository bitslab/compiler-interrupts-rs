#![feature(thread_local)]

use anyhow::{Context, Result};
use nanorand::{Rng, WyRand};

const BASE_VAL: i32 = 10000;
const CI_INTERVAL: i64 = 1_000_000;
const MAX_THREADS: i32 = 8;

#[thread_local]
#[allow(non_upper_case_globals)]
static mut prev_ic: i64 = 0;

fn rand() -> i32 {
    let mut rng = WyRand::new();
    rng.generate_range(0..i32::MAX)
}

fn interrupt_handler(ic: i64) {
    let interval;
    unsafe {
        interval = ic - prev_ic;
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

fn enable_hook() {
    println!("This function is called after CI is enabled");
}

fn disable_hook() {
    println!("This function is called before CI is disabled");
}

fn increment() -> Result<()> {
    unsafe {
        println!("interval: {}", CI_INTERVAL);
        compiler_interrupts::register(CI_INTERVAL, CI_INTERVAL, interrupt_handler);
    }

    let mut counter = 0;
    let iterations = BASE_VAL + (rand() % 10);
    for _ in 0..iterations {
        counter += rand() % 10;
    }
    println!(
        "increment(): thread: {} -> counter: {}",
        std::thread::current()
            .name()
            .context("failed to get thread name")?,
        counter
    );

    Ok(())
}

fn decrement() -> Result<()> {
    unsafe {
        println!("interval: {}", CI_INTERVAL);
        compiler_interrupts::register(CI_INTERVAL, CI_INTERVAL, interrupt_handler);
    }

    // register the enable and disable hooks
    unsafe {
        // `enable_hook` will be called when enable the CI
        compiler_interrupts::register_enable_hook(enable_hook);

        // `disable_hook` will be called when disable the CI
        compiler_interrupts::register_disable_hook(disable_hook);
    }

    // temporarily disable CI for the remaining part of the execution
    unsafe {
        compiler_interrupts::disable();
    }

    let mut counter = 0;
    let iterations = BASE_VAL + (rand() % 10);

    unsafe {
        // do nothing since CI is already disabled
        compiler_interrupts::disable();

        // as there were two disable calls before
        // we have to call enable twice to re-enable CI
        compiler_interrupts::enable();
        compiler_interrupts::enable();
    }

    for _ in 0..iterations {
        counter -= rand() % 10;
    }

    // de-register CI, which effectively disable CI for
    // the remaining part of the execution
    unsafe {
        compiler_interrupts::deregister();
    }

    println!(
        "decrement(): thread: {} -> counter: {}",
        std::thread::current()
            .name()
            .context("failed to get thread name")?,
        counter
    );

    Ok(())
}

fn main() -> Result<()> {
    // register the CI handler
    unsafe {
        compiler_interrupts::register(CI_INTERVAL, CI_INTERVAL, interrupt_handler);
    }

    // check argument for number of threads
    let args = std::env::args().collect::<Vec<_>>();
    let num_threads = if args.len() == 2 {
        let n = args[1].parse().unwrap_or(MAX_THREADS);
        if n > MAX_THREADS {
            println!("max threads: {}", MAX_THREADS);
            MAX_THREADS
        } else {
            n
        }
    } else {
        MAX_THREADS
    };

    // increment threads
    println!("starting {} increment threads", num_threads);
    let mut threads = vec![];
    for thread_id in 0..num_threads {
        let thread = std::thread::Builder::new()
            .name(format!("inc{}", thread_id))
            .spawn(|| increment())
            .context("failed to create thread")?;
        threads.push(thread);
    }
    for thread in threads {
        thread.join().expect("thread panicked")?;
    }

    // decrement threads
    println!("starting {} decrement threads", num_threads);
    let mut threads = vec![];
    for thread_id in 0..num_threads {
        let thread = std::thread::Builder::new()
            .name(format!("dec{}", thread_id))
            .spawn(|| decrement())
            .context("failed to create thread")?;
        threads.push(thread);
    }
    for thread in threads {
        thread.join().expect("thread panicked")?;
    }

    Ok(())
}
