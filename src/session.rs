use {
    crate::queue::SESSION_BUF_SIZE,
    anyhow::Result,
    crossterm::event::KeyEvent,
    ratatui::{
        layout::{Alignment, Rect},
        style::{Color, Style},
        widgets::{Block, Paragraph},
        Frame,
    },
    ringbuf::{traits::Consumer, wrap::CachingCons},
};

type SessionCons = CachingCons<ringbuf::Arc<ringbuf::StaticRb<u8, SESSION_BUF_SIZE>>>;

pub struct Session {
    cons: SessionCons,
    log: String,
}

impl Session {
    pub fn new(cons: SessionCons) -> Self {
        Self {
            cons,
            log: String::new(),
        }
    }

    pub fn poll_ring(&mut self) {
        while let Some(b) = self.cons.try_pop() {
            self.log.push(b as char);
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

    pub fn handle_key(&mut self, _key: KeyEvent) -> Result<()> {
        //toggle modes, hotkeys etc.
        Ok(())
    }
}
