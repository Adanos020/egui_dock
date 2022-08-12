use super::utils::*;
use egui::style::Margin;
use egui::*;

/// Specifies the look and feel of egui_dock.
pub struct Style {
    pub padding: Option<Margin>,

    pub background_color: Color32,
    pub border_color: Color32,
    pub border_size: f32,
    pub selection_color: Color32,
    pub separator_size: f32,
    pub separator_extra: f32,
    pub separator_color: Color32,

    pub tab_bar_background: Color32,

    pub tab_text: Color32,
    pub tab_outline_color: Color32,
    pub tab_rounding: Rounding,
    pub tab_background_color: Color32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            padding: Default::default(),

            border_color: Default::default(),
            border_size: Default::default(),

            background_color: Color32::WHITE,
            selection_color: Color32::from_rgb_additive(0, 92, 128),
            separator_size: 1.0,
            separator_extra: 100.0,
            separator_color: Color32::default(),

            tab_bar_background: Color32::WHITE,

            tab_text: Color32::BLACK,
            tab_outline_color: Color32::BLACK,
            tab_background_color: Color32::WHITE,
            tab_rounding: Default::default(),
        }
    }
}

impl Style {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by `egui::Style` are: `selection`, `background`, `tab_bar_background`, `tab_text`,
    /// `tab_outline`, and `tab_background`.
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            selection_color: style.visuals.selection.bg_fill.linear_multiply(0.5),

            background_color: style.visuals.window_fill(),
            tab_bar_background: style.visuals.faint_bg_color,

            separator_color: style.visuals.faint_bg_color,
            border_color: style.visuals.faint_bg_color,
            tab_text: style.visuals.widgets.active.fg_stroke.color,
            tab_outline_color: style.visuals.widgets.active.bg_stroke.color,
            tab_background_color: style.visuals.widgets.active.bg_fill,

            ..Self::default()
        }
    }

    pub(crate) fn hsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = midpoint - self.separator_size * 0.5;
        separator.max.x = midpoint + self.separator_size * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeHorizontal);

        {
            let delta = response.drag_delta().x;
            let range = rect.max.x - rect.min.x;
            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = map_to_pixel(
            midpoint - self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.x = map_to_pixel(
            midpoint + self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_right_of(separator.max.x)),
            separator,
            rect.intersect(Rect::everything_left_of(separator.min.x)),
        )
    }

    pub(crate) fn vsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = midpoint - self.separator_size * 0.5;
        separator.max.y = midpoint + self.separator_size * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeVertical);

        {
            let delta = response.drag_delta().y;
            let range = rect.max.y - rect.min.y;

            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = map_to_pixel(
            midpoint - self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.y = map_to_pixel(
            midpoint + self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_above(separator.min.y)),
            separator,
            rect.intersect(Rect::everything_below(separator.max.y)),
        )
    }

    pub(crate) fn tab_title(
        &self,
        ui: &mut Ui,
        label: String,
        active: bool,
        is_being_dragged: bool,
    ) -> Response {
        let px = ui.ctx().pixels_per_point().recip();
        let rounding = self.tab_rounding;

        let font_id = FontId::proportional(14.0);
        let galley = ui.painter().layout_no_wrap(label, font_id, self.tab_text);

        let offset = egui::vec2(8.0, 0.0);
        let text_size = galley.size();

        let mut desired_size = text_size + offset * 2.0;
        desired_size.y = 24.0;

        let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
        let response = response.on_hover_cursor(CursorIcon::PointingHand);

        match (active, is_being_dragged) {
            (true, false) => {
                let mut tab = rect;
                tab.min.x -= px;
                tab.max.x += px;
                ui.painter()
                    .rect_filled(tab, rounding, self.tab_outline_color);

                tab.min.x += px;
                tab.max.x -= px;
                tab.min.y += px;
                ui.painter()
                    .rect_filled(tab, rounding, self.background_color);
            }
            (true, true) => {
                let tab = rect;

                ui.painter().rect_stroke(
                    tab,
                    self.tab_rounding,
                    Stroke::new(1.0, self.tab_outline_color),
                );
            }
            _ => (),
        }

        let pos = Align2::LEFT_TOP
            .anchor_rect(rect.shrink2(vec2(8.0, 5.0)))
            .min;

        ui.painter().galley(pos, galley);

        response
    }
}

#[derive(Default)]
pub struct StyleBuilder {
    style: Style,
}

impl StyleBuilder {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn with_padding(mut self, padding: Option<Margin>) -> Self {
        self.style.padding = padding;
        self
    }

    #[inline(always)]
    pub fn with_background_color(mut self, background_color: Color32) -> Self {
        self.style.background_color = background_color;
        self
    }

    #[inline(always)]
    pub fn with_border_color(mut self, border_color: Color32) -> Self {
        self.style.border_color = border_color;
        self
    }

    #[inline(always)]
    pub fn with_border_width(mut self, border_width: f32) -> Self {
        self.style.border_size = border_width;
        self
    }

    #[inline(always)]
    pub fn with_selection_color(mut self, selection_color: Color32) -> Self {
        self.style.selection_color = selection_color;
        self
    }

    #[inline(always)]
    pub fn with_separator_size(mut self, separator_size: f32) -> Self {
        self.style.separator_size = separator_size;
        self
    }

    #[inline(always)]
    pub fn with_separator_color(mut self, separator_color: Color32) -> Self {
        self.style.separator_color = separator_color;
        self
    }

    #[inline(always)]
    pub fn with_tab_bar_background(mut self, tab_bar_background: Color32) -> Self {
        self.style.tab_bar_background = tab_bar_background;
        self
    }

    #[inline(always)]
    pub fn with_tab_text(mut self, tab_text: Color32) -> Self {
        self.style.tab_text = tab_text;
        self
    }

    #[inline(always)]
    pub fn with_tab_outline_color(mut self, tab_outline_color: Color32) -> Self {
        self.style.tab_outline_color = tab_outline_color;
        self
    }

    #[inline(always)]
    pub fn with_tab_rounding(mut self, tab_rounding: Rounding) -> Self {
        self.style.tab_rounding = tab_rounding;
        self
    }

    #[inline(always)]
    pub fn with_tab_background_color(mut self, tab_background: Color32) -> Self {
        self.style.tab_background_color = tab_background;
        self
    }

    #[inline(always)]
    pub fn build(self) -> Style {
        self.style
    }
}
