const BASE_VAL: i32 = 10000;
const MAX_THREADS: i32 = 64;

static mut PREVIOUS_IC: i64 = 0;

fn rand() -> i32 {
    use nanorand::{Rng, WyRand};
    let mut rng = WyRand::new();
    rng.generate_range(0..i32::MAX)
}

fn interrupt_handler(ic: i64) {
    unsafe {
        println!(
            "CI @ {}: last interval = {} IR",
            std::thread::current().name().unwrap(),
            ic - PREVIOUS_IC
        );
        PREVIOUS_IC = ic;
    }
}

fn enable_hook() {
    println!("This function is called after CI is enabled");
}

fn disable_hook() {
    println!("This function is called before CI is disabled");
}

fn increment(thread_id: i32) {
    // register the CI handler for 10000 IR and cycles interval
    unsafe {
        compiler_interrupts::register(10000, 10000, interrupt_handler);
    }

    let mut counter = 0;
    let iterations = BASE_VAL + (rand() % 10);
    for _ in 0..iterations {
        counter += rand() % 10;
    }
    println!("increment(): thread: {} -> counter: {}", thread_id, counter);
}

fn decrement(thread_id: i32) {
    // register the CI handler for 10000 IR and cycles interval
    unsafe {
        compiler_interrupts::register(10000, 10000, interrupt_handler);
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

    // de-register CI, which effectively disable CI for
    // the remaining part of the execution
    unsafe {
        compiler_interrupts::deregister();
    }

    for _ in 0..iterations {
        counter -= rand() % 10;
    }
    println!("decrement(): thread: {} -> counter: {}", thread_id, counter);
}

fn main() {
    // register the CI handler for 1000 IR and cycles interval
    unsafe {
        compiler_interrupts::register(1000, 1000, interrupt_handler);
    }

    // check for number of threads argument
    let args = std::env::args().collect::<Vec<_>>();
    let num_threads = if args.len() == 2 {
        let n = args[1].parse().unwrap_or(MAX_THREADS);
        if n > MAX_THREADS {
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
            .spawn(move || increment(thread_id))
            .unwrap();
        threads.push(thread);
    }
    for thread in threads {
        thread.join().expect("failed to join thread");
    }

    // decrement threads
    println!("starting {} decrement threads", num_threads);
    let mut threads = vec![];
    for thread_id in 0..num_threads {
        let thread = std::thread::Builder::new()
            .name(format!("dec{}", thread_id))
            .spawn(move || decrement(thread_id))
            .unwrap();
        threads.push(thread);
    }
    for thread in threads {
        thread.join().expect("failed to join thread");
    }
}
