use crossterm::cursor::{MoveRight, SetCursorStyle::BlinkingBar};
use std::io::{Result, Stdout, Write, stdout};

pub struct Output {
    pub text: String,
    pub length: usize,
    pub pos: usize,
    pub start: usize,
    pub replace: bool,
    out: Stdout,
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.out.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.out.flush()
    }
}

/// Constructor
impl Output {
    pub fn new() -> Self {
        let mut out = stdout();
        let _ = write!(out, "{BlinkingBar}");

        Self {
            text: String::new(),
            length: 0,
            pos: 0,
            start: 0,
            replace: false,
            out,
        }
    }
}
/// Utilities
impl Output {
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }
    pub const fn is_at_start(&self) -> bool {
        self.pos == 0
    }
    pub const fn is_at_end(&self) -> bool {
        self.pos == self.length
    }
    pub fn char_pos(&self, pos: usize) -> (usize, usize) {
        let (pos, char) = self
            .text
            .char_indices()
            .nth(pos)
            .expect("Character position out of range!");
        (pos, char.len_utf8())
    }
    pub fn get_pos(pos: usize) -> (usize, usize) {
        let w = super::terminal_width();
        let absolute_pos = pos + 2;
        (absolute_pos % w, absolute_pos / w)
    }
    pub fn get_current_pos(&self) -> (usize, usize) {
        Self::get_pos(self.pos)
    }
    pub fn get_end(&self) -> (usize, usize) {
        Self::get_pos(self.length)
    }
    pub fn cursor_to(&mut self, to: usize) -> Result<()> {
        if to > 0 {
            write!(self.out, "\r{}", MoveRight(to as u16))
        } else {
            write!(self.out, "\r")
        }
    }
    pub fn cursor_to_relative(&mut self, to: usize) -> Result<()> {
        write!(
            self.out,
            "\r{}",
            MoveRight((to.saturating_sub(self.start) + 2) as u16)
        )
    }
}
