use crate::utils::*;
use egui::{style::Margin, *};

/// Left or right alignment for tab add button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[allow(missing_docs)]
pub enum TabAddAlign {
    Left,
    Right,
}

/// Specifies the look and feel of egui_dock.
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Style {
    pub default_inner_margin: Margin,

    /// Sets padding to indent from the edges of the window. By `Default` it's `None`.
    pub dock_area_padding: Option<Margin>,

    /// Sets selection color for the placing area of the tab where this tab targeted on it.
    /// By `Default` it's `(0, 191, 255)` (light blue) with `0.5` capacity.
    pub selection_color: Color32,

    pub border: Stroke,
    pub buttons: Buttons,
    pub separator: Separator,
    pub tab_bar: TabBar,
    pub tabs: Tabs,
}

#[derive(Clone, Debug)]
pub struct Buttons {
    /// Color of the close tab button.
    pub close_tab_color: Color32,

    /// Color of the active close tab button.
    pub close_tab_active_color: Color32,

    /// Color of the background close tab button.
    pub close_tab_bg_fill: Color32,

    /// Left or right aligning of the add tab button.
    pub add_tab_align: TabAddAlign,

    /// Color of the add tab button.
    pub add_tab_color: Color32,

    /// Color of the active add tab button.
    pub add_tab_active_color: Color32,

    /// Color of the background add tab button.
    pub add_tab_bg_fill: Color32,
}

#[derive(Clone, Debug)]
pub struct Separator {
    /// Width of the rectangle separator between nodes. By `Default` it's `1.0`.
    pub width: f32,

    /// Limit for the allowed area for the separator offset. By `Default` it's `175.0`.
    /// `bigger value > less allowed offset` for the current window size.
    pub extra: f32,

    /// Idle color of the rectangle separator. By `Default` it's [`Color32::BLACK`].
    pub color_idle: Color32,

    /// Hovered color of the rectangle separator. By `Default` it's [`Color32::GRAY`].
    pub color_hovered: Color32,

    /// Dragged color of the rectangle separator. By `Default` it's [`Color32::WHITE`].
    pub color_dragged: Color32,
}

#[derive(Clone, Debug)]
pub struct TabBar {
    /// Background color of tab bar. By `Default` it's [`Color32::WHITE`].
    pub bg_fill: Color32,

    /// Height of the tab bar. By `Default` it's `24.0`.
    pub height: f32,
}

#[derive(Clone, Debug)]
pub struct Tabs {
    /// Color of the outline around tabs. By `Default` it's [`Color32::BLACK`].
    pub outline_color: Color32,

    /// Tab rounding. By `Default` it's [`Rounding::default`]
    pub rounding: Rounding,

    /// Colour of the tab's background. By `Default` it's [`Color32::WHITE`]
    pub bg_fill: Color32,

    /// Color of tab title when an inactive tab is unfocused.
    pub text_color_unfocused: Color32,

    /// Color of tab title when an inactive tab is focused.
    pub text_color_focused: Color32,

    /// Color of tab title when an active tab is unfocused.
    pub text_color_active_unfocused: Color32,

    /// Color of tab title when an active tab is focused.
    pub text_color_active_focused: Color32,

    /// Color of th line separating the tab name area from the tab content area.
    /// By `Default` it's [`Color32::BLACK`].
    pub hline_color: Color32,

    /// If `true`, show the hline below the active tabs name.
    /// If `false`, show the active tab as merged with the tab ui area.
    /// By `Default` it's `false`.
    pub hline_below_active_tab_name: bool,

    /// Whether tab titles expand to fill the width of their tab bars.
    pub fill_tab_bar: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            dock_area_padding: None,
            default_inner_margin: Margin::same(4.0),
            border: Stroke::new(f32::default(), Color32::BLACK),
            selection_color: Color32::from_rgb(0, 191, 255).linear_multiply(0.5),
            buttons: Buttons::default(),
            separator: Separator::default(),
            tab_bar: TabBar::default(),
            tabs: Tabs::default(),
        }
    }
}

impl Default for Buttons {
    fn default() -> Self {
        Self {
            close_tab_color: Color32::WHITE,
            close_tab_active_color: Color32::WHITE,
            close_tab_bg_fill: Color32::GRAY,

            add_tab_align: TabAddAlign::Right,
            add_tab_color: Color32::WHITE,
            add_tab_active_color: Color32::WHITE,
            add_tab_bg_fill: Color32::GRAY,
        }
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self {
            width: 1.0,
            extra: 175.0,
            color_idle: Color32::BLACK,
            color_hovered: Color32::GRAY,
            color_dragged: Color32::WHITE,
        }
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self {
            bg_fill: Color32::WHITE,
            height: 24.0,
        }
    }
}

impl Default for Tabs {
    fn default() -> Self {
        Self {
            bg_fill: Color32::WHITE,
            fill_tab_bar: false,
            hline_color: Color32::BLACK,
            hline_below_active_tab_name: false,
            outline_color: Color32::BLACK,
            rounding: Rounding::default(),
            text_color_unfocused: Color32::DARK_GRAY,
            text_color_focused: Color32::BLACK,
            text_color_active_unfocused: Color32::DARK_GRAY,
            text_color_active_focused: Color32::BLACK,
        }
    }
}

impl Style {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`Style::border`]
    /// - [`Style::selection_color`]
    /// - [`Buttons::close_tab_bg_fill`]
    /// - [`Buttons::close_tab_color`]
    /// - [`Buttons::close_tab_active_color`]
    /// - [`Buttons::add_tab_bg_fill`]
    /// - [`Buttons::add_tab_color`]
    /// - [`Buttons::add_tab_active_color`]
    /// - [`Separator::color_idle`]
    /// - [`Separator::color_hovered`]
    /// - [`Separator::color_dragged`]
    /// - [`TabBar::bg_fill`]
    /// - [`Tabs::outline_color`]
    /// - [`Tabs::hline_color`]
    /// - [`Tabs::bg_fill`]
    /// - [`Tabs::text_color_unfocused`]
    /// - [`Tabs::text_color_focused`]
    /// - [`Tabs::text_color_active_unfocused`]
    /// - [`Tabs::text_color_active_focused`]
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            border: Stroke {
                color: style.visuals.widgets.active.bg_fill,
                ..Stroke::default()
            },
            selection_color: style.visuals.selection.bg_fill.linear_multiply(0.5),
            buttons: Buttons {
                close_tab_bg_fill: style.visuals.widgets.active.bg_fill,
                close_tab_color: style.visuals.text_color(),
                close_tab_active_color: style.visuals.strong_text_color(),
                add_tab_bg_fill: style.visuals.widgets.active.bg_fill,
                add_tab_color: style.visuals.text_color(),
                add_tab_active_color: style.visuals.strong_text_color(),
                ..Buttons::default()
            },
            separator: Separator {
                // Same as egui panel resize colors:
                color_idle: style.visuals.widgets.noninteractive.bg_stroke.color, // dim
                color_hovered: style.visuals.widgets.hovered.fg_stroke.color,     // bright
                color_dragged: style.visuals.widgets.active.fg_stroke.color,      // bright
                ..Separator::default()
            },
            tab_bar: TabBar {
                bg_fill: style.visuals.faint_bg_color,
                ..TabBar::default()
            },
            tabs: Tabs {
                outline_color: style.visuals.widgets.active.bg_fill,
                hline_color: style.visuals.widgets.active.bg_fill,
                bg_fill: style.visuals.window_fill(),
                text_color_unfocused: style.visuals.text_color(),
                text_color_focused: style.visuals.strong_text_color(),
                text_color_active_unfocused: style.visuals.text_color(),
                text_color_active_focused: style.visuals.strong_text_color(),
                ..Tabs::default()
            },
            ..Self::default()
        }
    }

    pub(crate) fn hsplit(
        &self,
        ui: &mut Ui,
        fraction: &mut f32,
        rect: Rect,
    ) -> (Response, Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = midpoint - self.separator.width * 0.5;
        separator.max.x = midpoint + self.separator.width * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal);

        if let Some(pos) = response.interact_pointer_pos() {
            let x = pos.x;
            let delta = response.drag_delta().x;

            if (delta > 0. && x > midpoint && x < rect.max.x)
                || (delta < 0. && x < midpoint && x > rect.min.x)
            {
                let range = rect.max.x - rect.min.x;
                let min = (self.separator.extra / range).min(1.0);
                let max = 1.0 - min;
                let (min, max) = (min.min(max), max.max(min));
                *fraction = (*fraction + delta / range).clamp(min, max);
            }
        }

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = map_to_pixel(
            midpoint - self.separator.width * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.x = map_to_pixel(
            midpoint + self.separator.width * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            response,
            rect.intersect(Rect::everything_right_of(separator.max.x)),
            separator,
            rect.intersect(Rect::everything_left_of(separator.min.x)),
        )
    }

    pub(crate) fn vsplit(
        &self,
        ui: &mut Ui,
        fraction: &mut f32,
        rect: Rect,
    ) -> (Response, Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = midpoint - self.separator.width * 0.5;
        separator.max.y = midpoint + self.separator.width * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_and_drag_cursor(CursorIcon::ResizeVertical);

        if let Some(pos) = response.interact_pointer_pos() {
            let y = pos.y;
            let delta = response.drag_delta().y;

            if (delta > 0. && y > midpoint && y < rect.max.y)
                || (delta < 0. && y < midpoint && y > rect.min.y)
            {
                let delta = response.drag_delta().y;
                let range = rect.max.y - rect.min.y;
                let min = (self.separator.extra / range).min(1.0);
                let max = 1.0 - min;
                let (min, max) = (min.min(max), max.max(min));
                *fraction = (*fraction + delta / range).clamp(min, max);
            }
        }

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = map_to_pixel(
            midpoint - self.separator.width * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.y = map_to_pixel(
            midpoint + self.separator.width * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            response,
            rect.intersect(Rect::everything_above(separator.min.y)),
            separator,
            rect.intersect(Rect::everything_below(separator.max.y)),
        )
    }

    pub(crate) const TAB_PLUS_SIZE: f32 = 24.0;

    pub(crate) fn tab_plus(&self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::splat(Self::TAB_PLUS_SIZE);

        let mut rect = ui.available_rect_before_wrap();

        match self.buttons.add_tab_align {
            TabAddAlign::Left => rect.max.x = rect.min.x + desired_size.x,
            TabAddAlign::Right => rect.min.x = rect.max.x - desired_size.x,
        }
        rect = rect.shrink(3.0);

        let rect = {
            let size = Self::TAB_PLUS_SIZE / 2.0;
            let mut pos = rect.right_top();
            pos.x -= size / 2.0;
            pos.y += rect.size().y / 2.0;
            Rect::from_center_size(pos, Vec2::splat(size))
        };

        let response = ui
            .allocate_rect(rect, Sense::hover())
            .on_hover_cursor(CursorIcon::PointingHand);

        let color = if response.hovered() {
            self.buttons.add_tab_active_color
        } else {
            self.buttons.add_tab_color
        };
        if response.hovered() {
            ui.painter().rect_filled(
                rect,
                Rounding::same(2.0),
                self.buttons.add_tab_bg_fill,
            );
        }

        let rect = rect.shrink(1.75);
        ui.painter().line_segment(
            [rect.center_top(), rect.center_bottom()],
            Stroke::new(1.0, color),
        );
        ui.painter().line_segment(
            [rect.right_center(), rect.left_center()],
            Stroke::new(1.0, color),
        );

        response
    }

    /// * `active` means "the tab that is opened in the parent panel".
    /// * `focused` means "the tab that was last interacted with".
    ///
    /// Returns the main button response plus the response of the close button, if any.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn tab_title(
        &self,
        ui: &mut Ui,
        label: WidgetText,
        focused: bool,
        active: bool,
        is_being_dragged: bool,
        id: Id,
        expanded_width: f32,
        show_close: bool,
    ) -> (Response, Option<Response>) {
        let rounding = self.tabs.rounding;

        let galley = label.into_galley(ui, None, f32::INFINITY, TextStyle::Button);

        let x_size = Vec2::splat(galley.size().y / 1.3);

        let offset = vec2(8.0, 0.0);

        let desired_size = if self.tabs.fill_tab_bar {
            vec2(expanded_width, self.tab_bar.height)
        } else if show_close {
            vec2(
                galley.size().x + offset.x * 2.0 + x_size.x + 5.0,
                self.tab_bar.height,
            )
        } else {
            vec2(
                galley.size().x + offset.x * 2.0,
                self.tab_bar.height,
            )
        };

        let (rect, mut response) = ui.allocate_at_least(desired_size, Sense::hover());
        if !ui.memory(|mem| mem.is_anything_being_dragged()) {
            response = response.on_hover_cursor(CursorIcon::Grab);
        }

        let (close_rect, close_response) =
            if (active || response.hovered()) && show_close {
                let mut pos = rect.right_top();
                pos.x -= offset.x + x_size.x / 2.0;
                pos.y += rect.size().y / 2.0;
                let x_rect = Rect::from_center_size(pos, x_size);
                let response = ui
                    .interact(x_rect, id, Sense::click())
                    .on_hover_cursor(CursorIcon::PointingHand);
                (x_rect, Some(response))
            } else {
                (Rect::NOTHING, None)
            };

        if active {
            if is_being_dragged {
                ui.painter().rect_stroke(
                    rect,
                    rounding,
                    Stroke::new(1.0, self.tabs.outline_color),
                );
            } else {
                let stroke = Stroke::new(1.0, self.tabs.outline_color);
                ui.painter()
                    .rect(rect, rounding, self.tabs.bg_fill, stroke);

                // Make the tab name area connect with the tab ui area:
                ui.painter().hline(
                    rect.x_range(),
                    rect.bottom(),
                    Stroke::new(2.0, self.tabs.bg_fill),
                );
            }
        }

        let text_pos = if self.tabs.fill_tab_bar {
            let mut pos = Align2::CENTER_CENTER.pos_in_rect(&rect.shrink2(vec2(8.0, 5.0)));
            pos -= galley.size() / 2.0;
            pos
        } else {
            let mut pos = Align2::LEFT_CENTER.pos_in_rect(&rect.shrink2(vec2(8.0, 5.0)));
            pos.y -= galley.size().y / 2.0;
            pos
        };

        let override_text_color = if galley.galley_has_color {
            None // respect the color the user has chosen
        } else {
            Some(match (active, focused) {
                (false, false) => self.tabs.text_color_unfocused,
                (false, true) => self.tabs.text_color_focused,
                (true, false) => self.tabs.text_color_active_unfocused,
                (true, true) => self.tabs.text_color_active_focused,
            })
        };

        ui.painter().add(epaint::TextShape {
            pos: text_pos,
            galley: galley.galley,
            underline: Stroke::NONE,
            override_text_color,
            angle: 0.0,
        });

        if (active || response.hovered()) && show_close {
            if close_response.as_ref().unwrap().hovered() {
                ui.painter().rect_filled(
                    close_rect,
                    Rounding::same(2.0),
                    self.buttons.close_tab_bg_fill,
                );
            }
            let x_rect = close_rect.shrink(1.75);

            let color = if focused
                || close_response
                    .as_ref()
                    .unwrap()
                    .interact_pointer_pos()
                    .is_some()
            {
                self.buttons.close_tab_active_color
            } else {
                self.buttons.close_tab_color
            };
            ui.painter().line_segment(
                [x_rect.left_top(), x_rect.right_bottom()],
                Stroke::new(1.0, color),
            );
            ui.painter().line_segment(
                [x_rect.right_top(), x_rect.left_bottom()],
                Stroke::new(1.0, color),
            );
        }

        (response, close_response)
        // match close_response {
        //     Some(some) => (response, some.hovered(), some.clicked()),
        //     None => (response, false, false),
        // }
    }
}

/// Builds a [`Style`] with custom configuration values.
#[derive(Default)]
#[deprecated]
pub struct StyleBuilder {
    style: Style,
}

#[allow(deprecated)]
impl StyleBuilder {
    #[inline(always)]
    /// Creates a new [`StyleBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Derives relevant fields from [`egui::Style`] and sets the remaining fields to their default values.
    ///
    /// See also: [`Style::from_egui`].
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            style: Style::from_egui(style),
        }
    }

    /// Sets padding to indent from the edges of the window. By `Default` it's `None`.
    #[inline(always)]
    pub fn with_padding(mut self, padding: Margin) -> Self {
        self.style.dock_area_padding = Some(padding);
        self
    }

    /// Sets border color of the window of "working area". By `Default` it's [`Color32::BLACK`].
    #[inline(always)]
    pub fn with_border_color(mut self, border_color: Color32) -> Self {
        self.style.border.color = border_color;
        self
    }

    /// Sets border width. By `Default` it's `0.0`.
    #[inline(always)]
    pub fn with_border_width(mut self, border_width: f32) -> Self {
        self.style.border.width = border_width;
        self
    }

    /// Sets color of the placing area of the tab where this tab targeted on it. By `Default` it's `(0, 191, 255)` (light blue) with `0.5` capacity.
    #[inline(always)]
    pub fn with_selection_color(mut self, selection_color: Color32) -> Self {
        self.style.selection_color = selection_color;
        self
    }

    /// Sets width of the rectangle separator between nodes. By `Default` it's `1.0`.
    #[inline(always)]
    pub fn with_separator_width(mut self, separator_width: f32) -> Self {
        self.style.separator.width = separator_width;
        self
    }

    /// Sets limit for the allowed area for the separator offset.
    /// By `Default` it's `175.0`. `bigger value > less allowed offset` for the current window size.
    #[inline(always)]
    pub fn with_separator_extra(mut self, separator_extra: f32) -> Self {
        self.style.separator.extra = separator_extra;
        self
    }

    /// Sets the idle color for the rectangle separator. By `Default` it's [`Color32::BLACK`].
    #[inline(always)]
    pub fn with_separator_color_idle(mut self, separator_color_idle: Color32) -> Self {
        self.style.separator.color_idle = separator_color_idle;
        self
    }

    /// Sets the hovered color for the rectangle separator. By `Default` it's [`Color32::GRAY`].
    #[inline(always)]
    pub fn with_separator_color_hovered(mut self, separator_color_hovered: Color32) -> Self {
        self.style.separator.color_hovered = separator_color_hovered;
        self
    }

    /// Sets the dragged color for the rectangle separator. By `Default` it's [`Color32::WHITE`].
    #[inline(always)]
    pub fn with_separator_color_dragged(mut self, separator_color_dragged: Color32) -> Self {
        self.style.separator.color_dragged = separator_color_dragged;
        self
    }

    /// Sets color of tab bar. By `Default` it's [`Color32::WHITE`].
    #[inline(always)]
    pub fn with_tab_bar_background(mut self, tab_bar_background_color: Color32) -> Self {
        self.style.tab_bar.bg_fill = tab_bar_background_color;
        self
    }

    /// Sets color of tab bar. By `Default` it's `24.0`.
    #[inline(always)]
    pub fn with_tab_bar_height(mut self, tab_bar_height: f32) -> Self {
        self.style.tab_bar.height = tab_bar_height;
        self
    }

    /// Sets color of tab outlines. By `Default` it's [`Color32::BLACK`].
    #[inline(always)]
    pub fn with_tab_outline_color(mut self, tab_outline_color: Color32) -> Self {
        self.style.tabs.outline_color = tab_outline_color;
        self
    }

    /// Sets color of the line separating the tab name area from the tab content area.
    ///
    /// By `Default` it's [`Color32::BLACK`].
    #[inline(always)]
    pub fn with_hline_color(mut self, hline_color: Color32) -> Self {
        self.style.tabs.hline_color = hline_color;
        self
    }

    /// If `true`, show the hline below the active tabs name.
    /// If `false`, show the active tab as merged with the tab ui area.
    /// 
    /// By `Default` it's `false`.
    #[inline(always)]
    pub fn with_hline_below_active_tab_name(mut self, hline_below_active_tab_name: bool) -> Self {
        self.style.tabs.hline_below_active_tab_name = hline_below_active_tab_name;
        self
    }

    /// Sets tab rounding.
    #[inline(always)]
    pub fn with_tab_rounding(mut self, tab_rounding: Rounding) -> Self {
        self.style.tabs.rounding = tab_rounding;
        self
    }

    /// Sets current tab background color.
    #[inline(always)]
    pub fn with_tab_background_color(mut self, tab_background: Color32) -> Self {
        self.style.tabs.bg_fill = tab_background;
        self
    }

    /// Sets close tab button color.
    #[inline(always)]
    pub fn with_close_tab_color(mut self, close_tab_color: Color32) -> Self {
        self.style.buttons.close_tab_color = close_tab_color;
        self
    }

    /// Sets active close tab button color.
    #[inline(always)]
    pub fn with_close_tab_active_color_color(mut self, close_tab_active_color: Color32) -> Self {
        self.style.buttons.close_tab_active_color = close_tab_active_color;
        self
    }

    /// Sets background close tab button color.
    #[inline(always)]
    pub fn with_close_tab_background_color_color(
        mut self,
        close_tab_background_color: Color32,
    ) -> Self {
        self.style.buttons.close_tab_bg_fill = close_tab_background_color;
        self
    }

    /// Sets add tab button.
    #[inline(always)]
    pub fn with_add_tab_align(mut self, add_tab_align: TabAddAlign) -> Self {
        self.style.buttons.add_tab_align = add_tab_align;
        self
    }

    /// Sets add tab button color.
    #[inline(always)]
    pub fn with_add_tab_color(mut self, add_tab_color: Color32) -> Self {
        self.style.buttons.add_tab_color = add_tab_color;
        self
    }

    /// Sets active add tab button color.
    #[inline(always)]
    pub fn with_add_tab_active_color_color(mut self, add_tab_active_color: Color32) -> Self {
        self.style.buttons.add_tab_active_color = add_tab_active_color;
        self
    }

    /// Sets background add tab button color.
    #[inline(always)]
    pub fn with_add_tab_background_color_color(
        mut self,
        add_tab_background_color: Color32,
    ) -> Self {
        self.style.buttons.add_tab_bg_fill = add_tab_background_color;
        self
    }

    /// Sets color of tab title when an inactive tab is unfocused.
    #[inline(always)]
    pub fn with_tab_text_color_unfocused(mut self, tab_text_color_unfocused: Color32) -> Self {
        self.style.tabs.text_color_unfocused = tab_text_color_unfocused;
        self
    }

    /// Sets color of tab title when an inactive tab is focused.
    #[inline(always)]
    pub fn with_tab_text_color_focused(mut self, tab_text_color_focused: Color32) -> Self {
        self.style.tabs.text_color_focused = tab_text_color_focused;
        self
    }

    /// Sets color of tab title when an active tab is unfocused.
    #[inline(always)]
    pub fn with_tab_text_color_active_unfocused(
        mut self,
        tab_text_color_active_unfocused: Color32,
    ) -> Self {
        self.style.tabs.text_color_active_unfocused = tab_text_color_active_unfocused;
        self
    }

    /// Sets color of tab title when an active tab is focused.
    #[inline(always)]
    pub fn with_tab_text_color_active_focused(
        mut self,
        tab_text_color_active_focused: Color32,
    ) -> Self {
        self.style.tabs.text_color_active_focused = tab_text_color_active_focused;
        self
    }

    /// Sets whether tab titles expand to fill the width of their tab bars.
    #[inline(always)]
    pub fn expand_tabs(mut self, expand_tabs: bool) -> Self {
        self.style.tabs.fill_tab_bar = expand_tabs;
        self
    }

    /// Returns [`Style`] with set values.
    #[inline(always)]
    pub fn build(self) -> Style {
        self.style
    }
}
