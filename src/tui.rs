use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, poll},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::{pty::ShellHost, queue::IoChannels, session::Session};

pub async fn run() -> Result<()> {
    let IoChannels {
        out_prod,
        out_cons,
        in_prod,
        in_cons,
    } = IoChannels::new();

    let pty = ShellHost::new(out_prod, in_cons);
    let mut session = Session::new(out_cons, in_prod);

    // start PTY on own OS thread
    let pty_handle = pty.spawn();

    //ratatui setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_tui_loop(&mut terminal, &mut session);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let _ = pty_handle.join();

    res
}

fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &mut Session,
) -> std::result::Result<(), anyhow::Error> {
    loop {
        // drain pty -> ring -> session state
        session.poll_ring();

        terminal.draw(|f| {
            let size = f.area();
            session.render(f, size);
        })?;

        // input handling
        if poll(Duration::from_micros(50))? {
            match event::read()? {
                event::Event::Key(k) => match k.code {
                    event::KeyCode::Char('q') => {
                        break;
                    }
                    _ => {
                        session.handle_key(k)?;
                    }
                },
                event::Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}
