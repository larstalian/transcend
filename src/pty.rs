use {
    crate::queue::{InCons, OutProd},
    anyhow::{Context, Result},
    portable_pty::{native_pty_system, CommandBuilder, PtySize},
    ringbuf::traits::{Consumer, Producer},
    std::{
        io::{Read, Write},
        thread::{self, JoinHandle},
        time::Duration,
    },
};

pub struct ShellHost {
    out_prod: OutProd,
    in_cons: InCons,
}

impl ShellHost {
    pub fn new(out_prod: OutProd, in_cons: InCons) -> Self {
        Self { out_prod, in_cons }
    }

    pub fn spawn(self) -> JoinHandle<Result<()>> {
        thread::spawn(move || self.run())
    }

    fn run(mut self) -> Result<()> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to open PTY")?;

        // spawn shell into slave
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
        let cmd = CommandBuilder::new(shell);

        pair.slave
            .spawn_command(cmd)
            .context("failed to spawn shell")?;

        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .context("failed to clone PTY reader")?;

        let mut writer = pair.master.take_writer().context("Failed to take writer")?;

        // spawn writer thread: session -> PTY input
        let mut in_cons = self.in_cons;
        let writer_handle = thread::spawn(move || -> Result<()> {
            loop {
                while let Some(b) = in_cons.try_pop() {
                    writer.write_all(&[b])?;
                }
                writer.flush()?;

                // avoid burning a full core
                thread::sleep(Duration::from_millis(5));
            }
        });

        let mut buf = [0u8; 4096];

        // main thread: PTY -> session output
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            self.out_prod.push_slice(&buf[..n]);
        }

        let _ = writer_handle.join();
        Ok(())
    }
}
