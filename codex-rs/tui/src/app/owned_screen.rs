//! Application-owned full-screen composition.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Clear;
use ratatui::widgets::Widget;

use crate::chatwidget::ChatWidget;
use crate::conversation_viewport::ConversationViewport;
use crate::history_cell::HistoryRenderMode;
use crate::key_hint::KeyBinding;
use crate::keymap::PagerKeymap;
use crate::render::renderable::Renderable;
use crate::sol_welcome::SolWelcome;
use crate::tui::FrameRequester;

pub(super) struct OwnedScreen {
    pub(super) viewport: ConversationViewport,
    welcome: SolWelcome,
}

impl OwnedScreen {
    pub(super) fn new(
        pager_keymap: PagerKeymap,
        frame_requester: FrameRequester,
        animations_enabled: bool,
        history_key: Option<KeyBinding>,
        shortcuts_key: Option<KeyBinding>,
    ) -> Self {
        Self {
            viewport: ConversationViewport::new(HistoryRenderMode::Rich, pager_keymap),
            welcome: SolWelcome::new(
                frame_requester,
                animations_enabled,
                history_key,
                shortcuts_key,
            ),
        }
    }

    pub(super) fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        chat_widget: &mut ChatWidget,
    ) -> Rect {
        Clear.render(area, buf);
        chat_widget.update_owned_screen_width(area.width);

        let bottom_pane = chat_widget.bottom_pane_renderable();
        let bottom_height = bottom_pane.desired_height(area.width).min(area.height);
        let canvas_height = area.height.saturating_sub(bottom_height);
        let canvas = Rect::new(area.x, area.y, area.width, canvas_height);
        let bottom = Rect::new(
            area.x,
            area.y.saturating_add(canvas_height),
            area.width,
            bottom_height,
        );

        let active_key = chat_widget.active_cell_transcript_key();
        self.viewport
            .set_render_mode(chat_widget.history_render_mode());
        self.viewport
            .sync_live_tail(area.width, active_key, |width| {
                chat_widget.active_cell_display_hyperlink_lines(width)
            });
        if self.viewport.is_empty() {
            self.welcome.render(canvas, buf);
        } else {
            self.viewport.render(canvas, buf);
        }

        bottom_pane.render(bottom, buf);
        chat_widget.note_rendered_width(area.width);
        bottom
    }
}
