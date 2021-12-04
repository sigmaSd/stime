use std::io::{Read, Write};
use stime::{
    advanced::{time_it, CustomLog, OUTPUT_TARGET},
    check, start,
};

#[derive(Default)]
struct Log {
    v: Vec<u8>,
    c: usize,
}
impl Write for Log {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.v.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.v.flush()
    }
}
impl Read for Log {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let n = buf.write(&self.v[self.c..])?;
        self.c += n;
        Ok(n)
    }
}

macro_rules! work {
    () => {
        let mut v = vec![0.to_string()];
        check!("vec init");
        for i in 0..100_000 {
            v.push(i.to_string());
        }
        check!("vec push 10_000 item");
    };
}
fn main() {
    {
        start!();
        work!();
        println!();
    }

    {
        let log: CustomLog<Log> = CustomLog::default();
        start!(@log.clone(), "start custom log");
        work!();
        println!("{}", log.read().unwrap());
    }

    {
        let f = {
            let f = std::fs::File::create(std::env::temp_dir().join("stime_example_log")).unwrap();
            let f = CustomLog::new(f);
            f
        };
        start!(@f, "start custom log file");
        work!();
        println!(
            "{}",
            std::fs::read_to_string(std::env::temp_dir().join("stime_example_log")).unwrap()
        );
    }

    {
        OUTPUT_TARGET.reset();
        let _g = time_it("time it");
        start!("Timeit");
        work!();
    }
}
