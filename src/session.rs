use {
    crate::queue::{InProd, OutCons},
    anyhow::Result,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    ratatui::{
        layout::{Alignment, Rect},
        style::{Color, Style},
        widgets::{Block, Paragraph},
        Frame,
    },
    ringbuf::traits::{Consumer, Producer},
};

pub struct Session {
    out_cons: OutCons,
    in_prod: InProd,
    log: String,
    in_escape: bool,
}

impl Session {
    pub fn new(out_cons: OutCons, in_prod: InProd) -> Self {
        Self {
            out_cons,
            in_prod,
            log: String::new(),
            in_escape: false,
        }
    }

    pub fn poll_ring(&mut self) {
        while let Some(b) = self.out_cons.try_pop() {
            self.push_byte_display(b);
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Transcend :: Shell")
            .border_style(Style::default().fg(Color::White));

        let content = Paragraph::new(self.log.as_str())
            .alignment(Alignment::Left)
            .block(block);

        frame.render_widget(content, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        use KeyCode::*;

        let mut buf: [u8; 8] = [0; 8];
        let mut len = 0;

        match (key.code, key.modifiers) {
            // Ctrl-C
            (Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                buf[0] = 0x03;
                len = 1;
            }
            // Normal printable chars
            (Char(ch), _) => {
                buf[0] = ch as u8;
                len = 1;
            }
            (Enter, _) => {
                buf[0] = b'\r';
                len = 1;
            }
            (Backspace, _) => {
                buf[0] = 0x7f;
                len = 1;
            }
            (Tab, _) => {
                buf[0] = b'\t';
                len = 1;
            }
            (Left, _) => {
                buf[..3].copy_from_slice(b"\x1b[D");
                len = 3;
            }
            (Right, _) => {
                buf[..3].copy_from_slice(b"\x1b[C");
                len = 3;
            }
            (Up, _) => {
                buf[..3].copy_from_slice(b"\x1b[A");
                len = 3;
            }
            (Down, _) => {
                buf[..3].copy_from_slice(b"\x1b[B");
                len = 3;
            }
            _ => {}
        }

        if len > 0 {
            let _ = self.in_prod.push_slice(&buf[..len]);
        }

        Ok(())
    }

    /// FIXME
    fn push_byte_display(&mut self, b: u8) {
        if self.in_escape {
            let c = b as char;
            if c.is_ascii_alphabetic() {
                self.in_escape = false;
            }
            return;
        }

        match b {
            0x1b => {
                self.in_escape = true;
            }
            b'\r' => {
                self.log.push('\n');
            }
            b'\n' => {}
            0x20..=0x7e => {
                self.log.push(b as char);
            }
            _ => {}
        }
    }
}
