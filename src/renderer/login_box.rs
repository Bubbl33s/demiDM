use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::lua_runtime::config::AxisPosition;
use crate::state::app_state::AppPhase;
use crate::state::{AppState, FocusTarget};

#[allow(dead_code)]
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

pub fn positioned_rect(
    width: u16,
    height: u16,
    area: Rect,
    x_pos: &AxisPosition,
    y_pos: &AxisPosition,
) -> Rect {
    let x = match x_pos {
        AxisPosition::Absolute(col) => area.x + (*col).min(area.width.saturating_sub(width)),
        AxisPosition::Fraction(frac) => {
            let offset = ((area.width.saturating_sub(width)) as f32 * frac) as u16;
            area.x + offset
        }
        AxisPosition::Named(named) => match named {
            crate::lua_runtime::config::NamedPosition::Center => {
                area.x + (area.width.saturating_sub(width)) / 2
            }
            crate::lua_runtime::config::NamedPosition::Start => area.x,
            crate::lua_runtime::config::NamedPosition::End => {
                area.x + area.width.saturating_sub(width)
            }
        },
    };

    let y = match y_pos {
        AxisPosition::Absolute(row) => area.y + (*row).min(area.height.saturating_sub(height)),
        AxisPosition::Fraction(frac) => {
            let offset = ((area.height.saturating_sub(height)) as f32 * frac) as u16;
            area.y + offset
        }
        AxisPosition::Named(named) => match named {
            crate::lua_runtime::config::NamedPosition::Center => {
                area.y + (area.height.saturating_sub(height)) / 2
            }
            crate::lua_runtime::config::NamedPosition::Start => area.y,
            crate::lua_runtime::config::NamedPosition::End => {
                area.y + area.height.saturating_sub(height)
            }
        },
    };

    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

pub fn draw_login_box(frame: &mut Frame, state: &AppState, area: Rect) {
    let theme = &state.config.theme;
    let login_box_config = &state.config.login_box;

    let box_rect = positioned_rect(
        login_box_config.width,
        12,
        area,
        &login_box_config.position.x,
        &login_box_config.position.y,
    );

    let border_type = login_box_config.border_style().to_border_type();
    let has_fb = state.fb_overlay.is_some();

    let bg = if has_fb { Color::Reset } else { Color::Black };

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_color))
        .border_type(border_type)
        .style(Style::default().bg(bg))
        .title(Span::styled(
            login_box_config.title.as_str(),
            Style::default()
                .fg(theme.foreground)
                .add_modifier(if theme.font_bold {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ));

    frame.render_widget(outer_block.clone(), box_rect);

    let inner = outer_block.inner(box_rect);
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);

    let _label_area = chunks[0];
    let username_area = chunks[1];
    let password_area = chunks[2];
    let status_area = chunks[3];

    let is_authenticating = matches!(state.phase, AppPhase::Authenticating);

    let username_focused = state.active_field == FocusTarget::Username && !is_authenticating;
    let password_focused = state.active_field == FocusTarget::Password && !is_authenticating;

    let username_border_color = if username_focused {
        theme.accent
    } else {
        theme.border_color
    };
    let password_border_color = if password_focused {
        theme.accent
    } else {
        theme.border_color
    };

    let username_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(username_border_color))
        .style(Style::default().bg(bg))
        .title(Span::styled(
            " Usuario ",
            Style::default().fg(theme.foreground),
        ));

    let username_text = state.username_field.display_value();
    let username_paragraph = Paragraph::new(username_text).block(username_block);
    frame.render_widget(username_paragraph, username_area);

    let password_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(password_border_color))
        .style(Style::default().bg(bg))
        .title(Span::styled(
            " Contrasena ",
            Style::default().fg(theme.foreground),
        ));

    let password_text = if is_authenticating {
        String::new()
    } else {
        state.password_field.masked_value('*')
    };
    let password_paragraph = Paragraph::new(password_text).block(password_block);
    frame.render_widget(password_paragraph, password_area);

    match &state.phase {
        AppPhase::Authenticating => {
            let status = Paragraph::new(Span::styled(
                " Authenticating...",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::ITALIC),
            ));
            frame.render_widget(status, status_area);
        }
        AppPhase::AuthFailure { message } => {
            let error_text = if message.is_empty() {
                " Authentication failed. Press any key.".to_string()
            } else {
                format!(" {} Press any key.", message)
            };
            let status = Paragraph::new(Span::styled(error_text, Style::default().fg(theme.error)));
            frame.render_widget(status, status_area);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render(state: &AppState) -> ratatui::buffer::Buffer {
        let area = Rect::new(0, 0, 80, 24);
        let backend = TestBackend::new(area.width, area.height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw_login_box(f, state, area)).unwrap();
        terminal.backend().buffer().clone()
    }

    #[test]
    fn test_theme_colors_propagate_to_rendered_border_and_title() {
        let mut state = AppState::new();
        state.config.theme.border_color = Color::Rgb(10, 20, 30);
        state.config.theme.foreground = Color::Rgb(40, 50, 60);
        state.config.login_box.title = "ZZTITLE".to_string();

        let buffer = render(&state);
        let area = Rect::new(0, 0, 80, 24);
        let box_rect = positioned_rect(
            state.config.login_box.width,
            12,
            area,
            &state.config.login_box.position.x,
            &state.config.login_box.position.y,
        );

        let corner_cell = buffer.get(box_rect.x, box_rect.y);
        assert_eq!(corner_cell.fg, Color::Rgb(10, 20, 30));

        let mut found_title = false;
        for x in box_rect.x..(box_rect.x + box_rect.width) {
            let cell = buffer.get(x, box_rect.y);
            if cell.symbol() == "Z" {
                assert_eq!(cell.fg, Color::Rgb(40, 50, 60));
                found_title = true;
                break;
            }
        }
        assert!(
            found_title,
            "title text was not found in the top border row"
        );
    }

    #[test]
    fn test_layout_responds_to_login_box_config() {
        let mut state = AppState::new();
        state.config.login_box.width = 30;
        state.config.login_box.position.x = AxisPosition::Absolute(2);
        state.config.login_box.position.y = AxisPosition::Absolute(3);

        let area = Rect::new(0, 0, 80, 24);
        let expected_rect = positioned_rect(
            state.config.login_box.width,
            12,
            area,
            &state.config.login_box.position.x,
            &state.config.login_box.position.y,
        );

        assert_eq!(expected_rect, Rect::new(2, 3, 30, 12));

        // Rendering at the computed rect should not panic and should place the
        // border's top-left corner exactly at the expected position.
        let buffer = render(&state);
        let corner_cell = buffer.get(expected_rect.x, expected_rect.y);
        assert_ne!(corner_cell.symbol(), " ");
    }

    #[test]
    fn test_password_field_is_always_masked() {
        let mut state = AppState::new();
        state.active_field = FocusTarget::Password;
        for c in "supersecret".chars() {
            state.password_field.push_char(c);
        }

        let buffer = render(&state);

        let mut combined = String::new();
        for cell in buffer.content() {
            combined.push_str(cell.symbol());
        }

        assert!(!combined.contains("supersecret"));
        assert!(combined.contains("***********"));
    }

    #[test]
    fn test_password_field_masked_while_authenticating() {
        let mut state = AppState::new();
        state.active_field = FocusTarget::Password;
        for c in "topsecret".chars() {
            state.password_field.push_char(c);
        }
        state.phase = AppPhase::Authenticating;

        let buffer = render(&state);

        let mut combined = String::new();
        for cell in buffer.content() {
            combined.push_str(cell.symbol());
        }

        assert!(!combined.contains("topsecret"));
    }
}
