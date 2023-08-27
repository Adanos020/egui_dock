use egui::{containers::*, emath::*, Align, Id, Key, Layout, Order, Response, Ui};

#[derive(Clone, Default, Debug)]
struct State {
    size: Vec2,
}

// This code was taken from the example here: https://github.com/emilk/egui/pull/1653#issuecomment-1133671051.
// It's needed because `egui`'s `popup_below_widget` currently doesn't respect window edges and will fall outside
// of them if near the right or bottom edge. It should be replaced with `egui` functions when this is fixed.
//
// All credit goes to https://github.com/zicklag.

/// Like `egui::popup_under_widget`, but pops up to the left, so that the popup doesn't go off the screen.
pub(crate) fn popup_under_widget<R>(
    ui: &Ui,
    popup_id: Id,
    widget_response: &Response,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    if !ui.memory(|mem| mem.is_popup_open(popup_id)) {
        return None;
    }

    let state: Option<State> = ui.data_mut(|d| d.get_temp(popup_id));

    // If this is the first draw, we don't know the popup size yet, so we don't know how to
    // position the popup
    if state.is_none() {
        ui.ctx().request_repaint();
    }

    let mut state = state.unwrap_or_default();

    let rect = Rect {
        min: widget_response.rect.left_bottom(),
        max: widget_response.rect.left_bottom() + state.size,
    };
    let inner = Area::new(popup_id)
        .order(Order::Foreground)
        .fixed_pos(constrain_window_rect_to_area(ui.ctx(), rect, None).min)
        .movable(true)
        .show(ui.ctx(), |ui| {
            // Note: we use a separate clip-rect for this area, so the popup can be outside the parent.
            // See https://github.com/emilk/egui/issues/825
            let frame = Frame::popup(ui.style());
            let frame_margin = frame.inner_margin + frame.outer_margin;
            let result = frame
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                        ui.set_width(widget_response.rect.width() - frame_margin.sum().x);
                        add_contents(ui)
                    })
                    .inner
                })
                .inner;

            state.size = ui.min_rect().size();

            result
        })
        .inner;

    ui.data_mut(|d| *d.get_temp_mut_or_default(popup_id) = state);

    if ui.input(|i| i.key_pressed(Key::Escape)) || widget_response.clicked_elsewhere() {
        ui.memory_mut(|mem| mem.close_popup());
    }
    Some(inner)
}

/// Copied egui because it is a private function on `egui::Context`
pub(crate) fn constrain_window_rect_to_area(
    ctx: &egui::Context,
    window: Rect,
    area: Option<Rect>,
) -> Rect {
    let mut area = area.unwrap_or_else(|| ctx.available_rect());

    if window.width() > area.width() {
        // Allow overlapping side bars.
        // This is important for small screens, e.g. mobiles running the web demo.
        area.max.x = ctx.input(|i| i.screen_rect()).max.x;
        area.min.x = ctx.input(|i| i.screen_rect()).min.x;
    }
    if window.height() > area.height() {
        // Allow overlapping top/bottom bars:
        area.max.y = ctx.input(|i| i.screen_rect()).max.y;
        area.min.y = ctx.input(|i| i.screen_rect()).min.y;
    }

    let mut pos = window.min;

    // Constrain to screen, unless window is too large to fit:
    let margin_x = (window.width() - area.width()).at_least(0.0);
    let margin_y = (window.height() - area.height()).at_least(0.0);

    pos.x = pos.x.at_most(area.right() + margin_x - window.width()); // move left if needed
    pos.x = pos.x.at_least(area.left() - margin_x); // move right if needed
    pos.y = pos.y.at_most(area.bottom() + margin_y - window.height()); // move right if needed
    pos.y = pos.y.at_least(area.top() - margin_y); // move down if needed

    Rect::from_min_size(pos, window.size())
}
