//! Retained conversation rendering for the application-owned full-screen viewport.

use std::cell::Cell;
use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;

use crate::chatwidget::ActiveCellTranscriptKey;
use crate::history_cell::HistoryCell;
use crate::history_cell::HistoryRenderMode;
use crate::history_cell::SessionHeaderHistoryCell;
use crate::history_cell::SessionInfoCell;
use crate::history_cell::UserHistoryCell;
use crate::keymap::PagerKeymap;
use crate::pager_overlay::PagerContent;
use crate::render::Insets;
use crate::render::renderable::InsetRenderable;
use crate::render::renderable::Renderable;
use crate::terminal_hyperlinks::HyperlinkLine;
use crate::terminal_hyperlinks::mark_buffer_hyperlinks;
use crate::terminal_hyperlinks::visible_lines_ref;
use crate::tui::ScrollDirection;
use crate::wrapping::RtOptions;
use crate::wrapping::adaptive_wrap_lines;

pub(crate) struct ConversationViewport {
    content: PagerContent,
    cells: Vec<Arc<dyn HistoryCell>>,
    render_mode: HistoryRenderMode,
    live_tail_key: Option<LiveTailKey>,
    live_tail_dismisses_welcome: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LiveTailKey {
    width: u16,
    revision: u64,
    is_stream_continuation: bool,
    animation_tick: Option<u64>,
}

impl ConversationViewport {
    pub(crate) fn new(render_mode: HistoryRenderMode, keymap: PagerKeymap) -> Self {
        Self {
            content: PagerContent::new(Vec::new(), keymap),
            cells: Vec::new(),
            render_mode,
            live_tail_key: None,
            live_tail_dismisses_welcome: false,
        }
    }

    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.content.render_bottom_aligned(area, buf);
    }

    pub(crate) fn render_over(&mut self, area: Rect, buf: &mut Buffer) {
        self.content.render_bottom_aligned_over(area, buf);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub(crate) fn dismisses_welcome(&self) -> bool {
        self.cells.iter().any(|cell| cell.dismisses_welcome()) || self.live_tail_dismisses_welcome
    }

    pub(crate) fn scroll(&mut self, direction: ScrollDirection) {
        self.content.scroll(direction, 3);
    }

    pub(crate) fn replace_cells(&mut self, cells: Vec<Arc<dyn HistoryCell>>) {
        let follow_bottom = self.content.is_following_bottom();
        self.take_live_tail_renderable();
        self.live_tail_key = None;
        self.cells = cells
            .into_iter()
            .filter(|cell| !is_session_metadata(cell.as_ref()))
            .collect();
        self.content
            .replace(Self::render_cells(&self.cells, self.render_mode));
        if follow_bottom {
            self.content.scroll_to_bottom();
        }
    }

    pub(crate) fn replace_cells_if_changed(&mut self, cells: &[Arc<dyn HistoryCell>]) {
        let visible = cells
            .iter()
            .filter(|cell| !is_session_metadata(cell.as_ref()))
            .cloned()
            .collect::<Vec<_>>();
        let unchanged = self.cells.len() == visible.len()
            && self
                .cells
                .iter()
                .zip(&visible)
                .all(|(current, next)| Arc::ptr_eq(current, next));
        if !unchanged {
            self.replace_cells(visible);
        }
    }

    pub(crate) fn set_render_mode(&mut self, render_mode: HistoryRenderMode) {
        if self.render_mode == render_mode {
            return;
        }
        let follow_bottom = self.content.is_following_bottom();
        self.take_live_tail_renderable();
        self.live_tail_key = None;
        self.render_mode = render_mode;
        self.content
            .replace(Self::render_cells(&self.cells, self.render_mode));
        if follow_bottom {
            self.content.scroll_to_bottom();
        }
    }

    pub(crate) fn sync_live_tail(
        &mut self,
        width: u16,
        active_key: Option<ActiveCellTranscriptKey>,
        dismisses_welcome: bool,
        compute_lines: impl FnOnce(u16) -> Option<Vec<HyperlinkLine>>,
    ) {
        let next_key = active_key.map(|key| LiveTailKey {
            width,
            revision: key.revision,
            is_stream_continuation: key.is_stream_continuation,
            animation_tick: key.animation_tick,
        });
        if self.live_tail_key == next_key && self.live_tail_dismisses_welcome == dismisses_welcome {
            return;
        }

        let follow_bottom = self.content.is_following_bottom();
        self.take_live_tail_renderable();
        self.live_tail_key = next_key;
        self.live_tail_dismisses_welcome = false;
        if let Some(key) = next_key {
            let lines = compute_lines(width).unwrap_or_default();
            if !lines.is_empty() {
                self.live_tail_dismisses_welcome = dismisses_welcome;
                self.content.push(Self::live_tail_renderable(
                    lines,
                    !self.cells.is_empty(),
                    key.is_stream_continuation,
                ));
            }
        }
        if follow_bottom {
            self.content.scroll_to_bottom();
        }
    }

    fn render_cells(
        cells: &[Arc<dyn HistoryCell>],
        render_mode: HistoryRenderMode,
    ) -> Vec<Box<dyn Renderable>> {
        cells
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                Self::cell_renderable(
                    cell.clone(),
                    render_mode,
                    /*has_prior_cells*/ index > 0,
                )
            })
            .collect()
    }

    fn cell_renderable(
        cell: Arc<dyn HistoryCell>,
        render_mode: HistoryRenderMode,
        has_prior_cells: bool,
    ) -> Box<dyn Renderable> {
        let is_stream_continuation = cell.is_stream_continuation();
        let renderable: Box<dyn Renderable> = Box::new(ConversationCellRenderable {
            cell,
            render_mode,
            cached_height: Cell::new(None),
        });
        if has_prior_cells && !is_stream_continuation {
            Self::with_leading_spacing(renderable)
        } else {
            renderable
        }
    }

    fn live_tail_renderable(
        lines: Vec<HyperlinkLine>,
        has_prior_cells: bool,
        is_stream_continuation: bool,
    ) -> Box<dyn Renderable> {
        let renderable: Box<dyn Renderable> = Box::new(HyperlinkLinesRenderable { lines });
        if has_prior_cells && !is_stream_continuation {
            Self::with_leading_spacing(renderable)
        } else {
            renderable
        }
    }

    fn with_leading_spacing(renderable: Box<dyn Renderable>) -> Box<dyn Renderable> {
        Box::new(InsetRenderable::new(
            renderable,
            Insets::tlbr(
                /*top*/ 1, /*left*/ 0, /*bottom*/ 0, /*right*/ 0,
            ),
        ))
    }

    fn take_live_tail_renderable(&mut self) -> Option<Box<dyn Renderable>> {
        (self.content.len() > self.cells.len()).then(|| self.content.pop())?
    }
}

pub(crate) fn is_session_metadata(cell: &dyn HistoryCell) -> bool {
    cell.as_any().is::<SessionHeaderHistoryCell>() || cell.as_any().is::<SessionInfoCell>()
}

struct ConversationCellRenderable {
    cell: Arc<dyn HistoryCell>,
    render_mode: HistoryRenderMode,
    cached_height: Cell<Option<(u16, u16)>>,
}

impl Renderable for ConversationCellRenderable {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if let Some(user) = self.cell.as_any().downcast_ref::<UserHistoryCell>() {
            Paragraph::new(owned_user_message_lines(user, area.width)).render(area, buf);
            return;
        }
        let hyperlink_lines = self
            .cell
            .display_hyperlink_lines_for_mode(area.width, self.render_mode);
        Paragraph::new(Text::from(visible_lines_ref(&hyperlink_lines)))
            .wrap(Wrap { trim: false })
            .render(area, buf);
        mark_buffer_hyperlinks(buf, area, &hyperlink_lines, /*scroll_rows*/ 0);
    }

    fn desired_height(&self, width: u16) -> u16 {
        if let Some((cached_width, height)) = self.cached_height.get()
            && cached_width == width
        {
            return height;
        }
        let height = if let Some(user) = self.cell.as_any().downcast_ref::<UserHistoryCell>() {
            owned_user_message_lines(user, width).len() as u16
        } else {
            self.cell.desired_height_for_mode(width, self.render_mode)
        };
        self.cached_height.set(Some((width, height)));
        height
    }
}

fn owned_user_message_lines(user: &UserHistoryCell, width: u16) -> Vec<Line<'static>> {
    const RAIL: &str = "│  ";
    let body_width = width.saturating_sub(RAIL.len() as u16).max(1);
    let text_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::ITALIC);
    let rail_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    adaptive_wrap_lines(
        user.raw_lines()
            .into_iter()
            .map(|line| line.style(text_style)),
        RtOptions::new(usize::from(body_width)).wrap_algorithm(textwrap::WrapAlgorithm::FirstFit),
    )
    .into_iter()
    .map(|line| {
        let mut spans = Vec::with_capacity(line.spans.len().saturating_add(1));
        spans.push(Span::styled(RAIL, rail_style));
        spans.extend(line.spans);
        Line::from(spans).style(text_style)
    })
    .collect()
}

struct HyperlinkLinesRenderable {
    lines: Vec<HyperlinkLine>,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::history_cell::AgentMarkdownCell;
    use crate::history_cell::UserHistoryCell;
    use crate::keymap::RuntimeKeymap;

    #[test]
    fn metadata_does_not_make_the_owned_canvas_nonempty() {
        let mut viewport =
            ConversationViewport::new(HistoryRenderMode::Rich, RuntimeKeymap::defaults().pager);
        let metadata: Arc<dyn HistoryCell> = Arc::new(SessionHeaderHistoryCell::new(
            "test-model".to_string(),
            None,
            false,
            Path::new("/tmp").to_path_buf(),
            "test",
        ));

        viewport.replace_cells(vec![metadata]);

        assert!(viewport.is_empty());
    }

    #[test]
    fn meaningful_conversation_hides_the_welcome() {
        let mut viewport =
            ConversationViewport::new(HistoryRenderMode::Rich, RuntimeKeymap::defaults().pager);
        let message: Arc<dyn HistoryCell> = Arc::new(AgentMarkdownCell::new(
            "hello from Sol".to_string(),
            Path::new("/tmp"),
        ));

        viewport.replace_cells(vec![message]);

        assert!(!viewport.is_empty());
        assert!(viewport.dismisses_welcome());
    }

    #[test]
    fn startup_warning_does_not_hide_the_welcome() {
        let mut viewport =
            ConversationViewport::new(HistoryRenderMode::Rich, RuntimeKeymap::defaults().pager);
        let warning: Arc<dyn HistoryCell> = Arc::new(crate::history_cell::new_warning_event(
            "MCP startup failed".to_string(),
        ));

        viewport.replace_cells(vec![warning]);

        assert!(!viewport.is_empty());
        assert!(!viewport.dismisses_welcome());
    }

    #[test]
    fn background_live_tail_does_not_hide_the_welcome() {
        let mut viewport =
            ConversationViewport::new(HistoryRenderMode::Rich, RuntimeKeymap::defaults().pager);
        let key = ActiveCellTranscriptKey {
            revision: 1,
            is_stream_continuation: false,
            animation_tick: None,
        };

        viewport.sync_live_tail(80, Some(key), /*dismisses_welcome*/ false, |_| {
            Some(vec![HyperlinkLine::from("Starting MCP servers")])
        });

        assert!(!viewport.is_empty());
        assert!(!viewport.dismisses_welcome());

        viewport.sync_live_tail(80, Some(key), /*dismisses_welcome*/ true, |_| {
            Some(vec![HyperlinkLine::from("Working")])
        });

        assert!(viewport.dismisses_welcome());
    }

    #[test]
    fn background_content_renders_over_the_welcome_without_clearing_it() {
        let mut viewport =
            ConversationViewport::new(HistoryRenderMode::Rich, RuntimeKeymap::defaults().pager);
        let warning: Arc<dyn HistoryCell> = Arc::new(crate::history_cell::new_warning_event(
            "MCP startup failed".to_string(),
        ));
        viewport.replace_cells(vec![warning]);
        let area = Rect::new(0, 0, 40, 6);
        let mut buffer = Buffer::empty(area);
        buffer[(0, 0)].set_symbol("W");

        viewport.render_over(area, &mut buffer);

        assert_eq!(buffer[(0, 0)].symbol(), "W");
        let rendered = buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("MCP startup failed"));
    }

    #[test]
    fn owned_user_messages_are_green_railed_and_bottom_aligned() {
        let mut viewport =
            ConversationViewport::new(HistoryRenderMode::Rich, RuntimeKeymap::defaults().pager);
        let message: Arc<dyn HistoryCell> = Arc::new(UserHistoryCell {
            message: "first\nsecond".to_string(),
            text_elements: Vec::new(),
            local_image_paths: Vec::new(),
            remote_image_urls: Vec::new(),
        });
        viewport.replace_cells(vec![message]);
        let area = Rect::new(0, 0, 20, 6);
        let mut buffer = Buffer::empty(area);

        viewport.render(area, &mut buffer);

        assert_eq!(buffer[(0, 4)].symbol(), "│");
        assert_eq!(buffer[(3, 4)].symbol(), "f");
        assert_eq!(buffer[(0, 5)].symbol(), "│");
        assert_eq!(buffer[(3, 5)].symbol(), "s");
        assert_eq!(buffer[(0, 4)].fg, Color::Green);
        assert_eq!(buffer[(3, 4)].fg, Color::Green);
        assert!(buffer[(3, 4)].modifier.contains(Modifier::ITALIC));
        assert_eq!(buffer[(0, 0)].symbol(), " ");
    }
}

impl Renderable for HyperlinkLinesRenderable {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::from(visible_lines_ref(&self.lines)))
            .wrap(Wrap { trim: false })
            .render(area, buf);
        mark_buffer_hyperlinks(buf, area, &self.lines, /*scroll_rows*/ 0);
    }

    fn desired_height(&self, width: u16) -> u16 {
        Paragraph::new(Text::from(visible_lines_ref(&self.lines)))
            .wrap(Wrap { trim: false })
            .line_count(width)
            .try_into()
            .unwrap_or(/*default*/ 0)
    }
}
