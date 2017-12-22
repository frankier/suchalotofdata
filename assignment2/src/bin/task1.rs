extern crate libc;
use std::collections::BTreeSet;
use std::time::Instant;
use std::cell::Cell;

thread_local!(static MEM_USAGE: Cell<Option<u64>> = Cell::new(None));

extern {
    fn je_stats_print (write_cb: extern fn (*const libc::c_void, *const libc::c_char), cbopaque: *const libc::c_void, opts: *const libc::c_char);
}

extern fn write_cb (_: *const libc::c_void, message: *const libc::c_char) {
    let stats = String::from_utf8_lossy(
        unsafe {
            std::ffi::CStr::from_ptr (message as *const i8) .to_bytes()
        }
    );
    let all = "Allocated: ";
    for line in stats.lines() {
        if stats.starts_with(all) {
            MEM_USAGE.with(|usage_cell| {
                usage_cell.set(Some(
                    line[all.len()..].split(',').next().unwrap().parse::<u64>().unwrap()
                ));
            });
            break;
        }
    }
}

fn get_mem_usage() -> u64 {
    unsafe {
        je_stats_print (write_cb, std::ptr::null(), std::ptr::null())
    };
    MEM_USAGE.with(|usage_cell| {
        usage_cell.get().unwrap()
    })
}

fn main() {
    // make an empty set
    let mut myset = BTreeSet::new();
    let mut i: u64 = 0;
    // while (true){
    loop {
        // start timer
        let start = Instant::now();
        // add 1000000 new elements to the set (you can just use 64 bit integers)
        for _ in 0..1000000 {
            myset.insert(i);
            i += 1;
        }
        // end timer and print time
        let elapsed = start.elapsed();
        let secs = elapsed.as_secs();
        let nanos = elapsed.subsec_nanos() as u64;
        println!("Time {}.{:09}", secs, nanos);
        println!("Mem {}", get_mem_usage());
    // }
    }
}
