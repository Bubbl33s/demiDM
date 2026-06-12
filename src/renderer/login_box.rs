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
