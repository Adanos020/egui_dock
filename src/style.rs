use super::utils::*;
use egui::{style::Margin, *};

/// Left or right alignment for tab add button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum TabAddAlign {
    Left,
    Right,
}

/// Specifies the look and feel of egui_dock.
#[derive(Clone, Debug)]
pub struct Style {
    pub dock_area_padding: Option<Margin>,
    pub default_inner_margin: Margin,

    pub border_color: Color32,
    pub border_width: f32,

    pub selection_color: Color32,

    pub separator_width: f32,
    pub separator_extra: f32,
    pub separator_color_idle: Color32,
    pub separator_color_hovered: Color32,
    pub separator_color_dragged: Color32,

    pub tab_bar_background_color: Color32,
    pub tab_bar_height: f32,

    pub tab_outline_color: Color32,
    pub tab_rounding: Rounding,
    pub tab_background_color: Color32,

    pub tab_text_color_unfocused: Color32,
    pub tab_text_color_focused: Color32,

    pub tabs_are_draggable: bool,
    pub expand_tabs: bool,

    pub close_tab_color: Color32,
    pub close_tab_active_color: Color32,
    pub close_tab_background_color: Color32,
    pub show_close_buttons: bool,

    pub add_tab_align: TabAddAlign,
    pub add_tab_color: Color32,
    pub add_tab_active_color: Color32,
    pub add_tab_background_color: Color32,
    pub show_add_buttons: bool,
    pub show_add_popup: bool,

    pub show_context_menu: bool,
    pub tab_include_scrollarea: bool,
    pub tab_hover_name: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            dock_area_padding: None,
            default_inner_margin: Margin::same(4.0),

            border_color: Color32::BLACK,
            border_width: Default::default(),

            selection_color: Color32::from_rgb(0, 191, 255).linear_multiply(0.5),
            separator_width: 1.0,
            separator_extra: 175.0,
            separator_color_idle: Color32::BLACK,
            separator_color_hovered: Color32::GRAY,
            separator_color_dragged: Color32::WHITE,

            tab_bar_background_color: Color32::WHITE,
            tab_bar_height: 24.0,

            tab_outline_color: Color32::BLACK,
            tab_rounding: Default::default(),
            tab_background_color: Color32::WHITE,

            tab_text_color_unfocused: Color32::DARK_GRAY,
            tab_text_color_focused: Color32::BLACK,

            close_tab_color: Color32::WHITE,
            close_tab_active_color: Color32::WHITE,
            close_tab_background_color: Color32::GRAY,
            show_close_buttons: true,

            add_tab_align: TabAddAlign::Right,
            add_tab_color: Color32::WHITE,
            add_tab_active_color: Color32::WHITE,
            add_tab_background_color: Color32::GRAY,
            show_add_buttons: false,
            show_add_popup: false,

            tabs_are_draggable: true,
            expand_tabs: false,
            show_context_menu: true,
            tab_include_scrollarea: true,
            tab_hover_name: false,
        }
    }
}

impl Style {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`Self::selection_color`]
    /// - [`Self::tab_bar_background_color`]
    /// - [`Self::tab_outline_color`]
    /// - [`Self::tab_background_color`]
    /// - [`Self::tab_text_color_unfocused`]
    /// - [`Self::tab_text_color_focused`]
    /// - [`Self::separator_color_idle`]
    /// - [`Self::separator_color_hovered`]
    /// - [`Self::separator_color_dragged`]
    /// - [`Self::border_color`]
    /// - [`Self::close_tab_background_color`]
    /// - [`Self::close_tab_color`]
    /// - [`Self::close_tab_active_color`]
    /// - [`Self::add_tab_background_color`]
    /// - [`Self::add_tab_color`]
    /// - [`Self::add_tab_active_color`]
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            selection_color: style.visuals.selection.bg_fill.linear_multiply(0.5),

            tab_bar_background_color: style.visuals.faint_bg_color,
            tab_outline_color: style.visuals.widgets.active.bg_fill,
            tab_background_color: style.visuals.window_fill(),

            tab_text_color_unfocused: style.visuals.text_color(),
            tab_text_color_focused: style.visuals.strong_text_color(),

            separator_color_idle: style.visuals.widgets.noninteractive.bg_stroke.color,
            separator_color_hovered: style.visuals.widgets.hovered.bg_stroke.color,
            separator_color_dragged: style.visuals.widgets.active.bg_stroke.color,

            border_color: style.visuals.widgets.active.bg_fill,

            close_tab_background_color: style.visuals.widgets.active.bg_fill,
            close_tab_color: style.visuals.text_color(),
            close_tab_active_color: style.visuals.strong_text_color(),

            add_tab_background_color: style.visuals.widgets.active.bg_fill,
            add_tab_color: style.visuals.text_color(),
            add_tab_active_color: style.visuals.strong_text_color(),
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

        match self.add_tab_align {
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
            self.add_tab_active_color
        } else {
            self.add_tab_color
        };
        if response.hovered() {
            ui.painter()
                .rect_filled(rect, Rounding::same(2.0), self.add_tab_background_color);
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

    /// `active` means "the tab that is opened in the parent panel".
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
    ) -> (Response, bool, bool) {
        let px = ui.ctx().pixels_per_point().recip();
        let rounding = self.tab_rounding;

        let galley = label.into_galley(ui, None, f32::INFINITY, TextStyle::Button);

        let x_size = Vec2::splat(galley.size().y / 1.3);

        let offset = vec2(8.0, 0.0);

        let desired_size = if self.expand_tabs {
            vec2(expanded_width, self.tab_bar_height)
        } else if self.show_close_buttons {
            vec2(
                galley.size().x + offset.x * 2.0 + x_size.x + 5.0,
                self.tab_bar_height,
            )
        } else {
            vec2(galley.size().x + offset.x * 2.0, self.tab_bar_height)
        };

        let (rect, mut response) = ui.allocate_at_least(desired_size, Sense::hover());
        if !ui.memory(|mem| mem.is_anything_being_dragged()) && is_being_dragged {
            response = response.on_hover_cursor(CursorIcon::Grab);
        }

        let (x_rect, x_res) = if (active || response.hovered()) && self.show_close_buttons {
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

        let pos = if self.expand_tabs {
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
        } else if focused {
            Some(self.tab_text_color_focused)
        } else {
            Some(self.tab_text_color_unfocused)
        };
        ui.painter().add(epaint::TextShape {
            pos,
            galley: galley.galley,
            underline: Stroke::NONE,
            override_text_color,
            angle: 0.0,
        });

        if (active || response.hovered()) && self.show_close_buttons {
            if x_res.as_ref().unwrap().hovered() {
                ui.painter().rect_filled(
                    x_rect,
                    Rounding::same(2.0),
                    self.close_tab_background_color,
                );
            }
            let x_rect = x_rect.shrink(1.75);

            let color = if focused || x_res.as_ref().unwrap().interact_pointer_pos().is_some() {
                self.close_tab_active_color
            } else {
                self.close_tab_color
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

        match x_res {
            Some(some) => (response, some.hovered(), some.clicked()),
            None => (response, false, false),
        }
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

    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// See also: [`Style::from_egui`].
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            style: Style::from_egui(style),
        }
    }

    /// Sets `padding` to indent from the edges of the window. By `Default` it's `None`.
    #[inline(always)]
    pub fn with_padding(mut self, padding: Margin) -> Self {
        self.style.dock_area_padding = Some(padding);
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

    /// Sets the idle color for the rectangle separator. By `Default` it's [`Color32::BLACK`].
    #[inline(always)]
    pub fn with_separator_color_idle(mut self, separator_color_idle: Color32) -> Self {
        self.style.separator_color_idle = separator_color_idle;
        self
    }

    /// Sets the hovered color for the rectangle separator. By `Default` it's [`Color32::GRAY`].
    #[inline(always)]
    pub fn with_separator_color_hovered(mut self, separator_color_hovered: Color32) -> Self {
        self.style.separator_color_hovered = separator_color_hovered;
        self
    }

    /// Sets the dragged color for the rectangle separator. By `Default` it's [`Color32::WHITE`].
    #[inline(always)]
    pub fn with_separator_color_dragged(mut self, separator_color_dragged: Color32) -> Self {
        self.style.separator_color_dragged = separator_color_dragged;
        self
    }

    /// Sets `tab_bar_background_color` for the color of tab bar. By `Default` it's [`Color32::WHITE`].
    #[inline(always)]
    pub fn with_tab_bar_background(mut self, tab_bar_background_color: Color32) -> Self {
        self.style.tab_bar_background_color = tab_bar_background_color;
        self
    }

    /// Sets `tab_bar_height` for the color of tab bar. By `Default` it's `24.0`.
    #[inline(always)]
    pub fn with_tab_bar_height(mut self, tab_bar_height: f32) -> Self {
        self.style.tab_bar_height = tab_bar_height;
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

    /// Sets `close_tab_color` for the close tab button color.
    #[inline(always)]
    pub fn with_close_tab_color(mut self, close_tab_color: Color32) -> Self {
        self.style.close_tab_color = close_tab_color;
        self
    }

    /// Sets `close_tab_active_color` for the active close tab button color.
    #[inline(always)]
    pub fn with_close_tab_active_color_color(mut self, close_tab_active_color: Color32) -> Self {
        self.style.close_tab_active_color = close_tab_active_color;
        self
    }

    /// Sets `close_tab_background_color` for the background close tab button color.
    #[inline(always)]
    pub fn with_close_tab_background_color_color(
        mut self,
        close_tab_background_color: Color32,
    ) -> Self {
        self.style.close_tab_background_color = close_tab_background_color;
        self
    }

    /// Shows / Hides the tab close buttons.
    #[inline(always)]
    pub fn show_close_buttons(mut self, show_close_buttons: bool) -> Self {
        self.style.show_close_buttons = show_close_buttons;
        self
    }

    /// Sets `add_tab_align` for the add tab button color.
    #[inline(always)]
    pub fn with_add_tab_align(mut self, add_tab_align: TabAddAlign) -> Self {
        self.style.add_tab_align = add_tab_align;
        self
    }

    /// Sets `add_tab_color` for the add tab button color.
    #[inline(always)]
    pub fn with_add_tab_color(mut self, add_tab_color: Color32) -> Self {
        self.style.add_tab_color = add_tab_color;
        self
    }

    /// Sets `add_tab_active_color` for the active add tab button color.
    #[inline(always)]
    pub fn with_add_tab_active_color_color(mut self, add_tab_active_color: Color32) -> Self {
        self.style.add_tab_active_color = add_tab_active_color;
        self
    }

    /// Sets `add_tab_background_color` for the background add tab button color.
    #[inline(always)]
    pub fn with_add_tab_background_color_color(
        mut self,
        add_tab_background_color: Color32,
    ) -> Self {
        self.style.add_tab_background_color = add_tab_background_color;
        self
    }

    /// Shows / Hides the tab add buttons.
    #[inline(always)]
    pub fn show_add_buttons(mut self, show_add_buttons: bool) -> Self {
        self.style.show_add_buttons = show_add_buttons;
        self
    }

    /// Shows / Hides the add button popup.
    #[inline(always)]
    pub fn show_add_popup(mut self, show_add_popup: bool) -> Self {
        self.style.show_add_popup = show_add_popup;
        self
    }

    /// Color of tab title when the tab is unfocused.
    #[inline(always)]
    pub fn with_tab_text_color_unfocused(mut self, tab_text_color_unfocused: Color32) -> Self {
        self.style.tab_text_color_unfocused = tab_text_color_unfocused;
        self
    }

    /// Color of tab title when the tab is focused.
    #[inline(always)]
    pub fn with_tab_text_color_focused(mut self, tab_text_color_focused: Color32) -> Self {
        self.style.tab_text_color_focused = tab_text_color_focused;
        self
    }

    /// Whether tabs can be dragged between nodes and reordered on the tab bar.
    #[inline(always)]
    pub fn tabs_are_draggable(mut self, tabs_are_draggable: bool) -> Self {
        self.style.tabs_are_draggable = tabs_are_draggable;
        self
    }

    /// Whether tab titles expand to fill the width of their tab bars.
    #[inline(always)]
    pub fn expand_tabs(mut self, expand_tabs: bool) -> Self {
        self.style.expand_tabs = expand_tabs;
        self
    }

    /// Whether tabs show a context menu.
    #[inline(always)]
    pub fn show_context_menu(mut self, show_context_menu: bool) -> Self {
        self.style.show_context_menu = show_context_menu;
        self
    }

    /// Whether tabs have a [`ScrollArea`](egui::containers::ScrollArea) out of the box.
    #[inline(always)]
    pub fn with_tab_scroll_area(mut self, tab_include_scrollarea: bool) -> Self {
        self.style.tab_include_scrollarea = tab_include_scrollarea;
        self
    }

    /// Wheter tabs show their name when hoverd over them.
    #[inline(always)]
    pub fn show_name_when_hovered(mut self, tab_hover_name: bool) -> Self {
        self.style.tab_hover_name = tab_hover_name;
        self
    }

    /// Returns `Style` with set values.
    #[inline(always)]
    pub fn build(self) -> Style {
        self.style
    }
}
