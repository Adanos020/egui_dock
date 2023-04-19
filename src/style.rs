use egui::{ecolor::*, Margin, Rounding, Stroke};

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
    pub buttons: ButtonsStyle,
    pub separator: SeparatorStyle,
    pub tab_bar: TabBarStyle,
    pub tabs: TabsStyle,
}

/// Specifies the look and feel of buttons.
#[derive(Clone, Debug)]
pub struct ButtonsStyle {
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

/// Specifies the look and feel of node separators.
#[derive(Clone, Debug)]
pub struct SeparatorStyle {
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

/// Specifies the look and feel of tab bars.
#[derive(Clone, Debug)]
pub struct TabBarStyle {
    /// Background color of tab bar. By `Default` it's [`Color32::WHITE`].
    pub bg_fill: Color32,

    /// Height of the tab bar. By `Default` it's `24.0`.
    pub height: f32,

    /// Show a scroll bar when tab bar overflows. By `Default` it's `true`.
    pub show_scroll_bar_on_overflow: bool,
}

/// Specifies the look and feel of individual tabs.
#[derive(Clone, Debug)]
pub struct TabsStyle {
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
            buttons: ButtonsStyle::default(),
            separator: SeparatorStyle::default(),
            tab_bar: TabBarStyle::default(),
            tabs: TabsStyle::default(),
        }
    }
}

impl Default for ButtonsStyle {
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

impl Default for SeparatorStyle {
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

impl Default for TabBarStyle {
    fn default() -> Self {
        Self {
            bg_fill: Color32::WHITE,
            height: 24.0,
            show_scroll_bar_on_overflow: true,
        }
    }
}

impl Default for TabsStyle {
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
    pub(crate) const TAB_ADD_BUTTON_SIZE: f32 = 24.0;
    pub(crate) const TAB_ADD_PLUS_SIZE: f32 = 12.0;
    pub(crate) const TAB_CLOSE_BUTTON_SIZE: f32 = 24.0;
    pub(crate) const TAB_CLOSE_X_SIZE: f32 = 9.0;
}

impl Style {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`Style::border`]
    /// - [`Style::selection_color`]
    /// - [`ButtonsStyle::close_tab_bg_fill`]
    /// - [`ButtonsStyle::close_tab_color`]
    /// - [`ButtonsStyle::close_tab_active_color`]
    /// - [`ButtonsStyle::add_tab_bg_fill`]
    /// - [`ButtonsStyle::add_tab_color`]
    /// - [`ButtonsStyle::add_tab_active_color`]
    /// - [`SeparatorStyle::color_idle`]
    /// - [`SeparatorStyle::color_hovered`]
    /// - [`SeparatorStyle::color_dragged`]
    /// - [`TabBarStyle::bg_fill`]
    /// - [`TabsStyle::outline_color`]
    /// - [`TabsStyle::hline_color`]
    /// - [`TabsStyle::bg_fill`]
    /// - [`TabsStyle::text_color_unfocused`]
    /// - [`TabsStyle::text_color_focused`]
    /// - [`TabsStyle::text_color_active_unfocused`]
    /// - [`TabsStyle::text_color_active_focused`]
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            border: Stroke {
                color: style.visuals.widgets.active.bg_fill,
                ..Stroke::default()
            },
            selection_color: style.visuals.selection.bg_fill.linear_multiply(0.5),
            buttons: ButtonsStyle {
                close_tab_bg_fill: style.visuals.widgets.active.bg_fill,
                close_tab_color: style.visuals.text_color(),
                close_tab_active_color: style.visuals.strong_text_color(),
                add_tab_bg_fill: style.visuals.widgets.active.bg_fill,
                add_tab_color: style.visuals.text_color(),
                add_tab_active_color: style.visuals.strong_text_color(),
                ..ButtonsStyle::default()
            },
            separator: SeparatorStyle {
                // Same as egui panel resize colors:
                color_idle: style.visuals.widgets.noninteractive.bg_stroke.color, // dim
                color_hovered: style.visuals.widgets.hovered.fg_stroke.color,     // bright
                color_dragged: style.visuals.widgets.active.fg_stroke.color,      // bright
                ..SeparatorStyle::default()
            },
            tab_bar: TabBarStyle {
                bg_fill: (Rgba::from(style.visuals.window_fill()) * Rgba::from_gray(0.7)).into(),
                ..TabBarStyle::default()
            },
            tabs: TabsStyle {
                outline_color: style.visuals.widgets.active.bg_fill,
                hline_color: style.visuals.widgets.active.bg_fill,
                bg_fill: style.visuals.window_fill(),
                text_color_unfocused: style.visuals.text_color(),
                text_color_focused: style.visuals.strong_text_color(),
                text_color_active_unfocused: style.visuals.text_color(),
                text_color_active_focused: style.visuals.strong_text_color(),
                ..TabsStyle::default()
            },
            ..Self::default()
        }
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
