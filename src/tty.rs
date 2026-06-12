use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

use nix::sys::termios::{self, Termios};

use crate::errors::AuraResult;

static ORIGINAL_TERMIOS: Mutex<Option<Termios>> = Mutex::new(None);

pub fn setup_tty(tty: &File) -> AuraResult<Termios> {
    let original = termios::tcgetattr(tty)?;

    let mut raw = original.clone();
    termios::cfmakeraw(&mut raw);
    termios::tcsetattr(tty, termios::SetArg::TCSANOW, &raw)?;

    let mut tty_writer = tty;
    write!(tty_writer, "\x1b[?25l")?;
    write!(tty_writer, "\x1b[?1049h")?;
    tty_writer.flush()?;

    if let Ok(mut guard) = ORIGINAL_TERMIOS.lock() {
        *guard = Some(original.clone());
    }

    Ok(original)
}

pub fn restore_tty(tty: &File, original: Termios) -> AuraResult<()> {
    let mut tty_writer = tty;
    write!(tty_writer, "\x1b[?25h")?;
    write!(tty_writer, "\x1b[?1049l")?;
    tty_writer.flush()?;

    termios::tcsetattr(tty, termios::SetArg::TCSANOW, &original)?;
    Ok(())
}

pub fn install_panic_hook(tty: File) {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(guard) = ORIGINAL_TERMIOS.lock() {
            if let Some(ref original) = *guard {
                let _ = restore_tty(&tty, original.clone());
            }
        }
        default_hook(info);
    }));
}

#[allow(dead_code)]
pub fn write_escape_codes<W: Write>(writer: &mut W, codes: &[&str]) -> std::io::Result<()> {
    for code in codes {
        write!(writer, "{}", code)?;
    }
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hide_cursor_sequence() {
        let mut buf = Vec::new();
        write_escape_codes(&mut buf, &["\x1b[?25l"]).unwrap();
        assert_eq!(buf, b"\x1b[?25l");
    }

    #[test]
    fn test_show_cursor_sequence() {
        let mut buf = Vec::new();
        write_escape_codes(&mut buf, &["\x1b[?25h"]).unwrap();
        assert_eq!(buf, b"\x1b[?25h");
    }

    #[test]
    fn test_enter_alternate_screen() {
        let mut buf = Vec::new();
        write_escape_codes(&mut buf, &["\x1b[?1049h"]).unwrap();
        assert_eq!(buf, b"\x1b[?1049h");
    }

    #[test]
    fn test_leave_alternate_screen() {
        let mut buf = Vec::new();
        write_escape_codes(&mut buf, &["\x1b[?1049l"]).unwrap();
        assert_eq!(buf, b"\x1b[?1049l");
    }

    #[test]
    fn test_setup_sequence() {
        let mut buf = Vec::new();
        write_escape_codes(&mut buf, &["\x1b[?25l", "\x1b[?1049h"]).unwrap();
        assert_eq!(buf, b"\x1b[?25l\x1b[?1049h");
    }

    #[test]
    fn test_restore_sequence() {
        let mut buf = Vec::new();
        write_escape_codes(&mut buf, &["\x1b[?25h", "\x1b[?1049l"]).unwrap();
        assert_eq!(buf, b"\x1b[?25h\x1b[?1049l");
    }
}
