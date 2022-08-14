use super::utils::*;
use egui::style::Margin;
use egui::*;

/// Specifies the look and feel of egui_dock.
#[derive(Clone)]
pub struct Style {
    pub padding: Option<Margin>,

    pub border_color: Color32,
    pub border_width: f32,
    pub selection_color: Color32,
    pub separator_width: f32,
    pub separator_extra: f32,
    pub separator_color: Color32,

    pub tab_bar_background_color: Color32,

    pub tab_outline_color: Color32,
    pub tab_rounding: Rounding,
    pub tab_background_color: Color32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            padding: Default::default(),

            border_color: Color32::BLACK,
            border_width: Default::default(),

            selection_color: Color32::from_rgb(0, 191, 255).linear_multiply(0.5),
            separator_width: 1.0,
            separator_extra: 175.0,
            separator_color: Color32::BLACK,

            tab_bar_background_color: Color32::WHITE,

            tab_outline_color: Color32::BLACK,
            tab_background_color: Color32::WHITE,
            tab_rounding: Default::default(),
        }
    }
}

impl Style {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are: `selection`, `tab_bar_background_color`, `tab_text`,
    /// `tab_outline_color`, `separator_color`, `border_color`, and `tab_background_color`.
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            selection_color: style.visuals.selection.bg_fill.linear_multiply(0.5),

            tab_bar_background_color: style.visuals.faint_bg_color,

            separator_color: style.visuals.faint_bg_color,
            border_color: style.visuals.faint_bg_color,
            tab_outline_color: style.visuals.widgets.active.bg_stroke.color,
            tab_background_color: style.visuals.window_fill(),

            ..Self::default()
        }
    }

    pub(crate) fn hsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = midpoint - self.separator_width * 0.5;
        separator.max.x = midpoint + self.separator_width * 0.5;

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
            midpoint - self.separator_width * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.x = map_to_pixel(
            midpoint + self.separator_width * 0.5,
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
        separator.min.y = midpoint - self.separator_width * 0.5;
        separator.max.y = midpoint + self.separator_width * 0.5;

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
            midpoint - self.separator_width * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.y = map_to_pixel(
            midpoint + self.separator_width * 0.5,
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
        label: WidgetText,
        active: bool,
        is_being_dragged: bool,
    ) -> Response {
        let px = ui.ctx().pixels_per_point().recip();
        let rounding = self.tab_rounding;

        let galley = label.into_galley(ui, None, 14.0, TextStyle::Button);

        let offset = vec2(8.0, 0.0);
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
                    .rect_filled(tab, rounding, self.tab_background_color);
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

        ui.painter().galley(pos, galley.galley);

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

    /// Sets `padding` to indent from the edges of the window. By `Default` it's `None`.  
    #[inline(always)]
    pub fn with_padding(mut self, padding: Option<Margin>) -> Self {
        self.style.padding = padding;
        self
    }

    /// Sets `border_color` for the window of "working area". By `Default` it's [`egui::Color32::BLACK`].
    #[inline(always)]
    pub fn with_border_color(mut self, border_color: Color32) -> Self {
        self.style.border_color = border_color;
        self
    }

    /// Sets `border_width` for the border. By `Default` it's `0.0`.
    #[inline(always)]
    pub fn with_border_width(mut self, border_width: f32) -> Self {
        self.style.border_width = border_width;
        self
    }

    /// Sets `selection color` for the placing area of the tab where this tab targeted on it. By `Default` it's `(0, 191, 255)` (light blue) with `0.5` capacity.
    #[inline(always)]
    pub fn with_selection_color(mut self, selection_color: Color32) -> Self {
        self.style.selection_color = selection_color;
        self
    }

    /// Sets `separator_size` for the rectangle separator between nodes. By `Default` it's `1.0`.
    #[inline(always)]
    pub fn with_separator_width(mut self, separator_width: f32) -> Self {
        self.style.separator_width = separator_width;
        self
    }

    /// Sets `separator_extra` it sets limit for the allowed area for the separator offset. By `Default` it's `175.0`.
    /// `bigger value > less allowed offset` for the current window size.
    #[inline(always)]
    pub fn with_separator_extra(mut self, separator_extra: f32) -> Self {
        self.style.separator_extra = separator_extra;
        self
    }

    /// Sets `separator_color`for the rectangle separator. By `Default` it's [`Color32::BLACK`].
    #[inline(always)]
    pub fn with_separator_color(mut self, separator_color: Color32) -> Self {
        self.style.separator_color = separator_color;
        self
    }

    /// Sets `tab_bar_background_color` for the color of tab bar. By `Default` it's [`Color32::WHITE`].
    #[inline(always)]
    pub fn with_tab_bar_background(mut self, tab_bar_background_color: Color32) -> Self {
        self.style.tab_bar_background_color = tab_bar_background_color;
        self
    }

    /// Sets `tab_outline_color` for the outline color of tabs. By `Default` it's [`Color32::BLACK`].
    #[inline(always)]
    pub fn with_tab_outline_color(mut self, tab_outline_color: Color32) -> Self {
        self.style.tab_outline_color = tab_outline_color;
        self
    }

    /// Sets `tab_rounding` for the tab rounding.
    #[inline(always)]
    pub fn with_tab_rounding(mut self, tab_rounding: Rounding) -> Self {
        self.style.tab_rounding = tab_rounding;
        self
    }

    /// Sets `tab_background_color` for the current tab background color.
    #[inline(always)]
    pub fn with_tab_background_color(mut self, tab_background: Color32) -> Self {
        self.style.tab_background_color = tab_background;
        self
    }

    /// Returns `Style` with set values.
    #[inline(always)]
    pub fn build(self) -> Style {
        self.style
    }
}
