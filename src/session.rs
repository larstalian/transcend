use {
    crate::queue::{InProd, OutCons},
    ratatui::{
        layout::{Alignment, Rect},
        style::{Color, Style},
        widgets::{Block, Paragraph},
        Frame,
    },
    ringbuf::traits::{Consumer, Producer},
};

const MAX_LINES: usize = 2000;

pub struct Session {
    out_cons: OutCons,
    in_prod: InProd,
    buf: TerminalBuffer,
    in_escape: bool,
}

struct TerminalBuffer {
    lines: Vec<String>,
    current: String,
}

impl Session {
    pub fn new(out_cons: OutCons, in_prod: InProd) -> Self {
        Self {
            out_cons,
            in_prod,
            buf: TerminalBuffer::new(),
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

        let content = Paragraph::new(self.buf.as_render_string())
            .alignment(Alignment::Left)
            .block(block);

        frame.render_widget(content, area);
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
                self.buf.new_line();
            }
            b'\n' => {}
            0x20..=0x7e => {
                self.buf.push_char(b as char);
            }
            _ => {}
        }
    }

    pub fn snapshot(&self, max_lines: usize) -> String {
        self.buf.tail(max_lines)
    }

    /// Inject to scrollback
    pub fn inject_line(&mut self, prefix: &str, text: &str) {
        self.buf.new_line();
        self.buf.push_str(prefix);
        self.buf.push_char(' ');
        self.buf.push_str(text);
        self.buf.new_line();
    }

    pub fn send_line(&mut self, line: &str) {
        if line.is_empty() {
            let _ = self.in_prod.push_slice(b"\r");
            return;
        }

        let mut bytes = line.as_bytes().to_vec();
        bytes.push(b'\r');
        let _ = self.in_prod.push_slice(&bytes);
    }
}

impl TerminalBuffer {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            current: String::new(),
        }
    }

    fn push_char(&mut self, ch: char) {
        self.current.push(ch);
    }

    fn push_str(&mut self, s: &str) {
        self.current.push_str(s);
    }

    fn new_line(&mut self) {
        let mut line = String::new();
        std::mem::swap(&mut line, &mut self.current);
        self.lines.push(line);
        if self.lines.len() > MAX_LINES {
            let excess = self.lines.len() - MAX_LINES;
            self.lines.drain(0..excess);
        }
    }

    /// TODO: more chad
    fn as_render_string(&self) -> String {
        let mut out = String::new();
        for line in &self.lines {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(&self.current);
        out
    }

    fn tail(&self, max_lines: usize) -> String {
        let start = self.lines.len().saturating_sub(max_lines);
        let mut out = String::new();
        for line in &self.lines[start..] {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(&self.current);
        out
    }
}
