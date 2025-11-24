use {
    crate::{pty::ShellHost, queue::IoChannels, session::Session},
    anyhow::Result,
    crossterm::{
        event::{self, poll, KeyCode, KeyEvent, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    ratatui::{
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout},
        widgets::{Block, Borders, Paragraph},
        Terminal,
    },
    std::{io, time::Duration},
};

pub struct UiState {
    ai: bool,
    line: String,
}

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
) -> Result<(), anyhow::Error> {
    let mut ui = UiState {
        ai: false,
        line: String::new(),
    };

    loop {
        // drain pty -> ring -> session state
        session.poll_ring();

        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(2)]) // 2 rows: border + text
                .split(size);

            // main area
            session.render(f, chunks[0]);

            // ----- input bar -----
            let prefix = if ui.ai { "[AI] " } else { "> " };
            let status = if ui.ai { "AI ON" } else { "AI OFF" };

            let bar_width = chunks[1].width as usize; // total cells in the bar

            let left = format!("{prefix}{}", ui.line);

            let left_len = left.chars().count();
            let status_len = status.chars().count();

            // 1 space between left and status
            let padding = bar_width.saturating_sub(left_len + 1 + status_len);

            let mut line = String::new();
            line.push_str(&left);
            if padding > 0 {
                line.push_str(&" ".repeat(padding));
            }
            line.push(' ');
            line.push_str(status);

            let input_block = Block::default().borders(Borders::TOP);
            let input_para = Paragraph::new(line).block(input_block);

            f.render_widget(input_para, chunks[1]);
        })?;

        // input handling
        if poll(Duration::from_micros(50))? {
            match event::read()? {
                event::Event::Key(k) => match k.code {
                    event::KeyCode::Char('q') => {
                        break;
                    }
                    _ => {
                        handle_key(k, session, &mut ui)?;
                    }
                },
                event::Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_key(key: KeyEvent, session: &mut Session, input: &mut UiState) -> Result<()> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        // control commands
    }

    match key.code {
        KeyCode::Backspace => {
            input.line.pop();
        }
        KeyCode::Enter if input.ai => {
            let prompt = std::mem::take(&mut input.line);

            let ctx = session.snapshot(200);

            session.inject_line("AI?", &prompt);

            let fake_answer = format!("(stub) ctx.len = {}, prompt = {:?}", ctx.len(), prompt);
            session.inject_line("[AI]", &fake_answer);
            input.ai = false;
        }
        KeyCode::Enter => {
            let cmd = std::mem::take(&mut input.line);
            session.send_line(&cmd);
        }
        // Printable chars
        KeyCode::Char(ch) => {
            input.line.push(ch);
        }
        KeyCode::Tab => {
            input.ai = !input.ai;
            return Ok(());
        }
        _ => {}
    }

    Ok(())
}
