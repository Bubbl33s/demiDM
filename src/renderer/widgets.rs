use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::app_state::AppState;
use crate::widget::WidgetInstance;

pub fn draw_widgets(frame: &mut Frame, state: &AppState) {
    let area = frame.size();
    let has_fb = state.fb_overlay.is_some();
    for widget in &state.widgets {
        draw_single_widget(frame, widget, area, has_fb);
    }
}

fn draw_single_widget(frame: &mut Frame, widget: &WidgetInstance, area: Rect, has_fb: bool) {
    let col = widget.def.position.col.min(area.width.saturating_sub(1));
    let row = widget.def.position.row.min(area.height.saturating_sub(1));
    let width = widget.def.width.min(area.width.saturating_sub(col));

    let height = match widget.def.height {
        Some(h) => h.min(area.height.saturating_sub(row)),
        None => {
            let content_lines = widget.content.lines().count() as u16;
            (content_lines + 2).min(area.height.saturating_sub(row))
        }
    };

    if width < 3 || height < 3 {
        return;
    }

    let rect = Rect::new(area.x + col, area.y + row, width, height);

    let border_type = widget.def.style.border.to_border_type();
    let bg = if has_fb {
        Color::Reset
    } else {
        widget.def.style.bg
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .style(Style::default().fg(widget.def.style.fg).bg(bg));

    let paragraph = Paragraph::new(widget.content.as_str()).block(block);
    frame.render_widget(paragraph, rect);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AbsolutePosition;
    use crate::renderer::theme::BorderStyle;
    use crate::widget::{WidgetDef, WidgetInstance, WidgetSource, WidgetStyle};
    use ratatui::style::Color;
    use std::time::Duration;

    fn make_widget(
        id: &str,
        col: u16,
        row: u16,
        width: u16,
        height: Option<u16>,
        content: &str,
    ) -> WidgetInstance {
        let mut instance = WidgetInstance::new(WidgetDef {
            id: id.to_string(),
            position: AbsolutePosition { col, row },
            width,
            height,
            refresh: Duration::from_secs(5),
            source: WidgetSource::StaticText(content.to_string()),
            style: WidgetStyle {
                border: BorderStyle::Plain,
                fg: Color::White,
                bg: Color::Black,
            },
        });
        instance.content = content.to_string();
        instance
    }

    #[test]
    fn test_draw_widgets_renders_without_panic() {
        let mut state = AppState::new();
        state
            .widgets
            .push(make_widget("w1", 2, 2, 20, Some(5), "hello"));

        let area = Rect::new(0, 0, 80, 24);
        let backend = ratatui::backend::TestBackend::new(area.width, area.height);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                draw_widgets(f, &state);
            })
            .unwrap();
    }

    #[test]
    fn test_draw_widget_auto_height() {
        let mut state = AppState::new();
        state
            .widgets
            .push(make_widget("w1", 0, 0, 20, None, "line1\nline2\nline3"));

        let area = Rect::new(0, 0, 80, 24);
        let backend = ratatui::backend::TestBackend::new(area.width, area.height);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                draw_widgets(f, &state);
            })
            .unwrap();
    }

    #[test]
    fn test_draw_widget_clipped_at_screen_edge() {
        let mut state = AppState::new();
        state
            .widgets
            .push(make_widget("w1", 78, 22, 20, Some(10), "test"));

        let area = Rect::new(0, 0, 80, 24);
        let backend = ratatui::backend::TestBackend::new(area.width, area.height);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                draw_widgets(f, &state);
            })
            .unwrap();
    }
}
