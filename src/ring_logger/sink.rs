// -- sink.rs --

use {
    chrono::Local,
    crossbeam_channel::Receiver,
    std::{
        fs::{File, OpenOptions},
        io::{self, Write},
        sync::atomic::{AtomicUsize, Ordering},
        thread::{self, JoinHandle},
    },
};

// --

static MSG_DROPED: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn drop_msg() {
    MSG_DROPED.fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn spawn(rx: Receiver<String>, mut name: String, limit: u64, roll: usize) -> JoinHandle<()> {
    thread::spawn(move || {
        if name.is_empty() {
            name = std::env::current_exe()
                .expect("current_exe() error")
                .file_name()
                .expect("file_name() error")
                .to_str()
                .expect("to_str() error")
                .to_string();
        }
        let mut f = FileSink::new(name, limit, roll);

        while let Ok(msg) = rx.recv() {
            f.write(msg.as_bytes()).expect("write to file error");
            if rx.is_empty() && MSG_DROPED.load(Ordering::Relaxed) > 0 {
                f.write_fmt(format_args!(
                    "{} {:<5} [{}:{}:{}] message droped: {}\n",
                    Local::now().format("%Y-%m-%d %H:%M:%S.%6f"),
                    log::Level::Warn.to_string(),
                    module_path!(),
                    file!(),
                    line!(),
                    MSG_DROPED.swap(0, Ordering::Relaxed)
                ))
                .expect("write message droped count to file error");
            }
        }
    })
}

// --

struct FileSink {
    name: String,
    limit: u64,
    roll: usize,
    current_roll: usize,
    file: File,
}

impl FileSink {
    fn new(name: String, limit: u64, roll: usize) -> Self {
        let current_roll = 0_usize;
        Self {
            name: name.clone(),
            limit: if limit == 0 { u64::max_value() } else { limit },
            roll: if roll == 0 { usize::max_value() } else { roll },
            current_roll,
            file: FileSink::create_file(&FileSink::roll_name(name, current_roll))
                .expect("create sinkfile error."),
        }
    }
    fn roll_name(mut name: String, roll: usize) -> String {
        name.push_str(&format!(".{}.txt", roll));
        name
    }
    fn create_file(name: &str) -> io::Result<File> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&name)
    }
    fn roll_file(&mut self) -> io::Result<()> {
        self.flush()?;
        self.current_roll += 1;
        self.current_roll %= self.roll;
        let n = FileSink::roll_name(self.name.clone(), self.current_roll);
        self.file = FileSink::create_file(&n)?;
        Ok(())
    }
    fn check(&mut self) -> io::Result<()> {
        let md = self.file.metadata()?;
        if md.len() > self.limit {
            self.roll_file()
        } else {
            Ok(())
        }
    }
}

impl Drop for FileSink {
    fn drop(&mut self) {
        self.flush().expect("flush file error");
    }
}

impl Write for FileSink {
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        self.check()?;
        self.file.write(msg)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
