// -- logger.rs --

use {
    super::sink::{drop_msg, spawn},
    chrono::Local,
    crossbeam_channel::{bounded, Sender, TrySendError},
    log::{Level, Log, Metadata, Record},
    std::{fmt::Write as FmtWrite, thread::JoinHandle},
};

// --

pub struct RingLogger {
    sink_handle: Option<JoinHandle<()>>,
}

impl RingLogger {
    pub fn new(cap: usize, level: Level, name: String, limit: u64, roll: usize) -> RingLogger {
        let (tx, rx) = bounded(cap);
        let sink_handle = spawn(rx, name, limit, roll);

        let logger = Logger { level, tx };
        log::set_boxed_logger(Box::new(logger)).expect("set_boxed_logger error");
        log::set_max_level(level.to_level_filter());
        log::info!("start of ring-logger");

        RingLogger {
            sink_handle: Some(sink_handle),
        }
    }
}

impl Drop for RingLogger {
    fn drop(&mut self) {
        log::info!("end of ring-logger");

        unsafe {
            let l = log::logger() as *const dyn Log as *const Logger as *mut Logger;
            Box::from_raw(l);
        }
        // 等待sink_spawn线程退出
        self.sink_handle.take().map(JoinHandle::join);
    }
}

// --

struct Logger {
    level: Level,
    tx: Sender<String>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut msg = String::new();
            writeln!(
                &mut msg,
                "{} {:<5} [{}:{}:{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S.%6f"),
                record.level().to_string(),
                record.target(),
                record.file().unwrap_or("<unknown file>"),
                record.line().unwrap_or(0),
                record.args()
            )
            .expect("writeln error");

            match self.tx.try_send(msg) {
                Err(TrySendError::Full(_)) => drop_msg(),
                Err(TrySendError::Disconnected(_)) => panic!("try_send disconnected"),
                _ => {}
            }
        }
    }

    fn flush(&self) {}
}
