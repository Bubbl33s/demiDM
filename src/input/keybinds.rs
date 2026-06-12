use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::events::AppEvent;

pub fn map_key_event(key_event: KeyEvent) -> Option<AppEvent> {
    match key_event.code {
        KeyCode::Esc => Some(AppEvent::Shutdown),
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppEvent::Shutdown)
        }
        _ => Some(AppEvent::KeyPress(key_event)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_esc_maps_to_shutdown() {
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(map_key_event(key_event), Some(AppEvent::Shutdown)));
    }

    #[test]
    fn test_ctrl_c_maps_to_shutdown() {
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(matches!(map_key_event(key_event), Some(AppEvent::Shutdown)));
    }

    #[test]
    fn test_tab_maps_to_keypress() {
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(matches!(
            map_key_event(key_event),
            Some(AppEvent::KeyPress(_))
        ));
    }

    #[test]
    fn test_char_maps_to_keypress() {
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(matches!(
            map_key_event(key_event),
            Some(AppEvent::KeyPress(_))
        ));
    }
}
