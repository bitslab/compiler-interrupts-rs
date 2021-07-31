#![feature(thread_local)]

#[cfg(target_os = "linux")]
mod profiler {
    use std::io::Write;

    use anyhow::{Context, Result};
    use nanorand::{Rng, WyRand};
    use nix::libc;
    use nix::sched::*;
    use nix::unistd::*;

    const BASE_VAL: i64 = 1_000_000;
    const MAX_THREADS: i64 = 2;
    static mut CI_INTERVAL: i64 = 10_000_000;

    #[thread_local]
    #[allow(non_upper_case_globals)]
    static mut prev_ic: i64 = 0;

    #[thread_local]
    #[allow(non_upper_case_globals)]
    static mut prev_tsc: u64 = 0;

    #[thread_local]
    #[allow(non_upper_case_globals)]
    static mut buffer_ic: Vec<i64> = Vec::new();

    #[thread_local]
    #[allow(non_upper_case_globals)]
    static mut buffer_tsc: Vec<u64> = Vec::new();

    fn rand() -> i64 {
        let mut rng = WyRand::new();
        rng.generate_range(0..i64::MAX)
    }

    fn interrupt_handler(curr_ic: i64) {
        unsafe {
            let ic = curr_ic - prev_ic;
            if ic < 0 {
                panic!("IR count was negative: {}", ic);
            }

            let mut aux: u32 = 0;
            let curr_tsc = std::arch::x86_64::__rdtscp(&mut aux);
            let tsc = curr_tsc - prev_tsc;

            buffer_ic.push(ic);
            buffer_tsc.push(tsc);

            prev_ic = curr_ic;
            prev_tsc = curr_tsc;
        }
    }

    fn log_intervals() -> Result<()> {
        unsafe {
            compiler_interrupts::deregister();
        }

        let thread = std::thread::current();
        let thread_name = thread.name().context("failed to get thread name")?;

        let filename = format!("{}_intervals.txt", thread_name);

        let len: usize;
        let ordered_ic: Vec<i64>;
        let ordered_tsc: Vec<u64>;
        unsafe {
            len = buffer_tsc.len();

            // sort both buffer_ic and buffer_tsc using buffer_tsc as the key
            let mut indicies: Vec<usize> = (0..len).collect();
            indicies.sort_by_key(|&i| buffer_tsc[i]);
            ordered_ic = indicies.iter().map(|&i| buffer_ic[i]).collect();
            ordered_tsc = indicies.iter().map(|&i| buffer_tsc[i]).collect();
        }

        let file = std::fs::File::create(filename)?;
        let mut buf = std::io::BufWriter::new(file);
        writeln!(buf, "percentile, time-stamp counter, instruction count\n")?;
        for i in 0..len {
            let percentage = i as f64 / (len - 1) as f64;
            writeln!(
                buf,
                "{:.5}, {}, {}\n",
                percentage, ordered_tsc[i], ordered_ic[i]
            )?;
        }

        unsafe {
            if !buffer_tsc.is_empty() {
                let i = buffer_tsc.len() / 2;
                println!(
                    "thread: {} -> median interval: {} cycles",
                    thread_name, buffer_tsc[i]
                );
            }
        }

        Ok(())
    }

    fn pin_thread() -> Result<()> {
        // POSIX standard doesn't define `_SC_NPROCESSORS_CONF`
        let var = unsafe { std::mem::transmute::<i32, SysconfVar>(libc::_SC_NPROCESSORS_CONF) };
        let max_cpus = sysconf(var)?.context("failed to get number of cpus")?;
        let cpu = gettid().as_raw() % (max_cpus as i32 - 1);
        let mut cpu_set = CpuSet::new();
        cpu_set.set(cpu as usize)?;
        sched_setaffinity(Pid::this(), &cpu_set)?;

        Ok(())
    }

    fn increment() -> Result<()> {
        pin_thread()?;

        unsafe {
            compiler_interrupts::register(CI_INTERVAL, CI_INTERVAL, interrupt_handler);
        }

        let mut counter = 0;
        let iterations = BASE_VAL + (rand() % 10);
        for _ in 0..iterations {
            counter += rand() % 10;
        }

        log_intervals()?;

        println!(
            "thread: {} -> counter: {}",
            std::thread::current()
                .name()
                .context("failed to get thread name")?,
            counter
        );

        Ok(())
    }

    pub fn main() -> Result<()> {
        pin_thread()?;

        // check argument for CI interval
        let args = std::env::args().collect::<Vec<_>>();
        if args.len() == 2 {
            unsafe {
                // safety: CI_INTERVAL is only modified once before other threads read it
                CI_INTERVAL = args[1].parse().unwrap_or(CI_INTERVAL);
                println!("Using interrupt interval: {} IR", CI_INTERVAL)
            }
        } else {
            unsafe {
                println!("Using default interrupt interval: {} IR", CI_INTERVAL);
                println!("To change the interval: ./profiler <interval>");
            }
        }

        unsafe {
            compiler_interrupts::register(CI_INTERVAL, CI_INTERVAL, interrupt_handler);
        }

        println!("starting {} increment threads", MAX_THREADS);
        let mut threads = vec![];
        for thread_id in 0..MAX_THREADS {
            let thread = std::thread::Builder::new()
                .name(format!("inc{}", thread_id))
                .spawn(|| increment())
                .expect("failed to create thread");
            threads.push(thread);
        }
        for thread in threads {
            thread.join().expect("thread panicked")?;
        }

        log_intervals()?;

        println!("Achieved intervals (in cycles) per thread are exported to *_intervals.txt files");

        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn main() -> anyhow::Result<()> {
    profiler::main()
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("`profiler` example is available only on x86-64 Linux platforms");
}
