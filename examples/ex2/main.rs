// -- main.rs --

#[macro_use]
extern crate servant_log;

// --

fn main() {
    init_ring_logger!(100, log::Level::Trace, String::new(), 1024*32, 5);

    log::debug!("this is a debug {}", "message");
    log::error!("this is printed by default");

    if log::log_enabled!(log::Level::Info) {
        let x = 3 * 4; // expensive computation
        log::info!("the answer was: {}", x);
    }

    for i in 0..2000 {
        log::info!("this is {}", i);
    }

    std::thread::sleep(std::time::Duration::from_millis(1000));
    for i in 0..300 {
        log::info!("this2 is {}", i);
    }


    example::test();
}

// --

mod example {
    pub fn test() {
        log::info!("from Example::test()");
    }
}
