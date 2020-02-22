// -- logger.rs --

use chrono::Local;
use log::{Level, Log, Metadata, Record};

use super::sink::spawn;
use ring_channel::{ring_channel, RingSender};
use std::{fmt::Write as FmtWrite, num::NonZeroUsize, sync::Mutex, thread::JoinHandle};

// --

pub struct RingLogger {
    sink_handle: Option<JoinHandle<()>>,
}

impl RingLogger {
    pub fn new(level: Level, name: String, limit: u64, roll: usize) -> RingLogger {
        const CAPICITY: usize = 10;
        let (tx, rx) = ring_channel(NonZeroUsize::new(CAPICITY).unwrap());
        let sink_handle = spawn(rx, name, limit, roll);

        let logger = Logger {
            level,
            tx: Mutex::new(tx),
        };
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(level.to_level_filter());
        log::info!("start of ring-logger");

        RingLogger { sink_handle: Some(sink_handle) }
    }
}

impl Drop for RingLogger {
    fn drop(&mut self) {
        log::info!("end of ring-logger");

        // 等待sink_spawn线程退出
        self.sink_handle.take().map(JoinHandle::join);
    }
}

// --

struct Logger {
    level: Level,
    tx: Mutex<RingSender<String>>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_string = { record.level().to_string() };
            let target = if !record.target().is_empty() {
                record.target()
            } else {
                record.module_path().unwrap_or_default()
            };
            let mut msg = String::new();
            writeln!(
                &mut msg,
                "{} {:<5} [{}:{}:{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S.%6f"),
                level_string,
                target,
                record.file().unwrap_or("<unknown file>"),
                record.line().unwrap_or(0),
                record.args()
            )
            .expect("writeln error");

            if let Ok(mut g) = self.tx.lock() {
                g.send(msg).expect("send error");
            }
        }
    }

    fn flush(&self) {}
}
