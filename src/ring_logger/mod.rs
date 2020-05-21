// -- mod.rs --

mod sink;
mod logger;

// --

#[macro_export]
macro_rules! init_ring_logger {
    ($cap:expr, $a:expr, $b:expr, $c:expr, $d:expr) => {
        let _ring_logger = servant_log::RingLogger::new($cap, $a, $b, $c, $d);
    };
}

// --

pub use logger::RingLogger;
