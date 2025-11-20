use {
    crate::queue::SessionProd,
    anyhow::{Context, Result},
    portable_pty::{native_pty_system, CommandBuilder, PtySize},
    ringbuf::traits::Producer,
    std::thread::{self, JoinHandle},
};

pub struct ShellHost {
    out_prod: SessionProd,
}

impl ShellHost {
    pub fn new(out_prod: SessionProd) -> Self {
        Self { out_prod }
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
        let cmd = CommandBuilder::new("bash"); // TODO: make configurable

        pair.slave
            .spawn_command(cmd)
            .context("failed to spawn shell")?;

        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .context("failed to clone PTY reader")?;

        let mut buf = [0u8; 4096];

        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }

            let _ = self.out_prod.push_slice(&buf[..n]);
        }

        Ok(())
    }
}
