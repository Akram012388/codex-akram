//! Empty-session welcome art for the application-owned full-screen canvas.

use std::time::Duration;
use std::time::Instant;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;

use crate::key_hint::KeyBinding;
use crate::render::highlight::foreground_style_for_scopes;
use crate::style::accent_style;
use crate::tui::FrameRequester;

const FRAME_INTERVAL: Duration = Duration::from_millis(80);
const WIDE_MIN_WIDTH: u16 = 92;
const WIDE_MIN_HEIGHT: u16 = 22;
const STACKED_MIN_WIDTH: u16 = 48;
const STACKED_MIN_HEIGHT: u16 = 18;
const TERMINAL_CELL_ASPECT_RATIO: u16 = 2;

pub(crate) struct SolWelcome {
    frame_requester: FrameRequester,
    animations_enabled: bool,
    started_at: Instant,
    history_key: Option<KeyBinding>,
    shortcuts_key: Option<KeyBinding>,
}

impl SolWelcome {
    pub(crate) fn new(
        frame_requester: FrameRequester,
        animations_enabled: bool,
        history_key: Option<KeyBinding>,
        shortcuts_key: Option<KeyBinding>,
    ) -> Self {
        Self {
            frame_requester,
            animations_enabled,
            started_at: Instant::now(),
            history_key,
            shortcuts_key,
        }
    }

    pub(crate) fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let phase = if self.animations_enabled {
            self.frame_requester.schedule_frame_in(FRAME_INTERVAL);
            self.started_at.elapsed().as_secs_f32() * 0.55
        } else {
            0.0
        };

        if area.width >= WIDE_MIN_WIDTH && area.height >= WIDE_MIN_HEIGHT {
            self.render_wide(area, buf, phase);
        } else if area.width >= STACKED_MIN_WIDTH && area.height >= STACKED_MIN_HEIGHT {
            self.render_stacked(area, buf, phase);
        } else {
            self.render_copy(centered_rect(area, area.width.min(40), 9), buf);
        }
    }

    fn render_wide(&self, area: Rect, buf: &mut Buffer, phase: f32) {
        let globe_height = 20;
        let globe_width = circular_width(globe_height, area.width);
        let copy_width = 40;
        let gap = 4;
        let width = globe_width + gap + copy_width;
        let height = globe_height.max(9);
        let content = centered_rect(area, width, height);
        let globe = Rect::new(content.x, content.y, globe_width, globe_height);
        let copy = Rect::new(
            globe.right().saturating_add(gap),
            content.y.saturating_add(5),
            copy_width.min(content.right().saturating_sub(globe.right() + gap)),
            10,
        );
        render_globe(globe, buf, phase);
        self.render_copy(copy, buf);
    }

    fn render_stacked(&self, area: Rect, buf: &mut Buffer, phase: f32) {
        let globe_height = area.height.saturating_sub(10).clamp(7, 13);
        let globe_width = circular_width(globe_height, area.width);
        let total_height = globe_height.saturating_add(10);
        let content = centered_rect(area, globe_width.max(40), total_height);
        let globe = Rect::new(
            content.x + content.width.saturating_sub(globe_width) / 2,
            content.y,
            globe_width,
            globe_height,
        );
        let copy = Rect::new(
            content.x,
            globe.bottom().saturating_add(1),
            content.width,
            9,
        );
        render_globe(globe, buf, phase);
        self.render_copy(copy, buf);
    }

    fn render_copy(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let history = self
            .history_key
            .map(welcome_key_label)
            .unwrap_or_else(|| "Ctrl+T".to_string());
        let shortcuts = self
            .shortcuts_key
            .map(welcome_key_label)
            .unwrap_or_else(|| "?".to_string());
        let title_style = accent_style().add_modifier(Modifier::BOLD);
        let key_style = Style::default().add_modifier(Modifier::BOLD);
        let muted = Style::default().add_modifier(Modifier::DIM);
        let quote_style = foreground_style_for_scopes(&["comment", "string.quoted"])
            .unwrap_or(muted)
            .add_modifier(Modifier::DIM);
        let lines = vec![
            Line::from(Span::styled("Welcome to Sol", title_style)),
            Line::default(),
            Line::from(vec![
                Span::styled(history, key_style),
                Span::styled("  for history", muted),
            ]),
            Line::from(vec![
                Span::styled(shortcuts, key_style),
                Span::styled("  for shortcuts", muted),
            ]),
            Line::default(),
            Line::from(Span::styled(
                "“Wait, Sol? AGI? I need to check this out.”",
                quote_style,
            )),
        ];
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

fn welcome_key_label(binding: KeyBinding) -> String {
    binding
        .display_label()
        .split(" + ")
        .map(|part| match part {
            "ctrl" => "Ctrl".to_string(),
            "alt" => "Alt".to_string(),
            "shift" => "Shift".to_string(),
            single if single.chars().count() == 1 => single.to_uppercase(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

/// Terminal cells are approximately twice as tall as they are wide. A visual circle therefore
/// occupies two columns for every row.
fn circular_width(height: u16, available_width: u16) -> u16 {
    height
        .saturating_mul(TERMINAL_CELL_ASPECT_RATIO)
        .min(available_width)
}

fn render_globe(area: Rect, buf: &mut Buffer, phase: f32) {
    if area.width < 4 || area.height < 3 {
        return;
    }
    let palette = globe_palette();
    let center_x = f32::from(area.x) + f32::from(area.width) / 2.0;
    let center_y = f32::from(area.y) + f32::from(area.height) / 2.0;
    let radius_x = f32::from(area.width) / 2.0;
    let radius_y = f32::from(area.height) / 2.0;

    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let nx = (f32::from(x) + 0.5 - center_x) / radius_x;
            let ny = (f32::from(y) + 0.5 - center_y) / radius_y;
            let distance = nx * nx + ny * ny;
            if distance > 1.0 {
                continue;
            }

            let depth = (1.0 - distance).sqrt();
            let longitude = nx.atan2(depth) + phase;
            let latitude = ny.asin();
            let ribbon = (longitude * 7.0 + latitude * 4.0).sin().abs();
            let shimmer = (longitude * 2.5 - latitude * 3.0).cos() * 0.14;
            let brightness = (depth * 0.58 + (1.0 - ny) * 0.17 + shimmer).clamp(0.0, 1.0);
            let density = (0.12 + brightness * 0.76 + (1.0 - ribbon) * 0.12).clamp(0.0, 0.94);
            if stable_noise(x, y) > density {
                continue;
            }

            let (symbol, level) = if brightness > 0.78 && stable_noise(y, x) > 0.42 {
                ("●", 3)
            } else if brightness > 0.57 {
                ("•", 2)
            } else if brightness > 0.34 {
                ("·", 1)
            } else {
                (".", 0)
            };
            let mut style = Style::default().fg(palette[level]);
            if level == 3 {
                style = style.add_modifier(Modifier::BOLD);
            }
            buf[(x, y)].set_symbol(symbol).set_style(style);
        }
    }
}

fn globe_palette() -> [Color; 4] {
    let dim = foreground_style_for_scopes(&["comment", "punctuation"])
        .and_then(|style| style.fg)
        .unwrap_or(Color::DarkGray);
    let cool = foreground_style_for_scopes(&["variable", "support.type"])
        .and_then(|style| style.fg)
        .unwrap_or(Color::Blue);
    let warm = foreground_style_for_scopes(&["string", "constant"])
        .and_then(|style| style.fg)
        .unwrap_or(Color::Green);
    let bright = accent_style().fg.unwrap_or(Color::LightGreen);
    [dim, cool, warm, bright]
}

fn stable_noise(x: u16, y: u16) -> f32 {
    let mut value = u32::from(x)
        .wrapping_mul(0x045d_9f3b)
        .wrapping_add(u32::from(y).wrapping_mul(0x27d4_eb2d));
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    f32::from((value & 0xffff) as u16) / f32::from(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_rect_stays_within_parent() {
        assert_eq!(
            centered_rect(Rect::new(10, 5, 20, 10), 8, 4),
            Rect::new(16, 8, 8, 4)
        );
        assert_eq!(
            centered_rect(Rect::new(2, 3, 5, 4), 20, 10),
            Rect::new(2, 3, 5, 4)
        );
    }

    #[test]
    fn globe_width_compensates_for_terminal_cell_aspect_ratio() {
        assert_eq!(circular_width(20, 100), 40);
        assert_eq!(circular_width(20, 32), 32);
    }

    #[test]
    fn stable_noise_is_stable_and_bounded() {
        let value = stable_noise(17, 29);
        assert_eq!(value, stable_noise(17, 29));
        assert!((0.0..=1.0).contains(&value));
    }

    #[test]
    fn welcome_shortcuts_use_compact_title_case_labels() {
        assert_eq!(
            welcome_key_label(crate::key_hint::ctrl(crossterm::event::KeyCode::Char('t'))),
            "Ctrl+T".to_string()
        );
    }
}
