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
    /// Sets padding to indent from the edges of the window. By `Default` it's `None`.
    pub dock_area_padding: Option<Margin>,

    /// Sets selection color for the placing area of the tab where this tab targeted on it.
    /// By `Default` it's `(0, 191, 255)` (light blue) with `0.5` capacity.
    pub selection_color: Color32,

    pub border: Stroke,
    pub rounding: Rounding,

    pub buttons: ButtonsStyle,
    pub separator: SeparatorStyle,
    pub tab_bar: TabBarStyle,
    pub tab: TabStyle,

    pub allowed_splits: SplitTypes,
}

/// What directions can this dock split in?
#[derive(Clone, Debug, Default)]
pub enum SplitTypes {
    #[default]
    /// Allow splits in any direction (horizontal and vertical).
    All,
    /// Only allow split in a horizontal direction.
    LeftRightOnly,
    /// Only allow splits in a vertical direction.
    TopBottomOnly,
    /// Don't allow splits at all.
    None,
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

    /// Color of the add tab button's background.
    pub add_tab_bg_fill: Color32,

    /// Color of the add tab button's left border.
    pub add_tab_border_color: Color32,
}

/// Specifies the look and feel of node separators.
#[derive(Clone, Debug)]
pub struct SeparatorStyle {
    /// Width of the rectangle separator between nodes. By `Default` it's `1.0`.
    pub width: f32,

    /// Extra width added to the "logical thickness" of the rectangle so it's
    /// easier to grab. By `Default` it's `4.0`.
    pub extra_interact_width: f32,

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

    /// Tab rounding. By `Default` it's [`Rounding::default`]
    pub rounding: Rounding,

    /// Color of th line separating the tab name area from the tab content area.
    /// By `Default` it's [`Color32::BLACK`].
    pub hline_color: Color32,

    /// Whether tab titles expand to fill the width of their tab bars.
    pub fill_tab_bar: bool,
}

/// Specifies the look and feel of an individual tab.
#[derive(Clone, Debug)]
pub struct TabStyle {
    /// Style of the tab when it is active.
    pub active: TabInteractionStyle,

    /// Style of the tab when it is inactive.
    pub inactive: TabInteractionStyle,

    /// Style of the tab when it is focused.
    pub focused: TabInteractionStyle,

    /// Style of the tab when it is hovered.
    pub hovered: TabInteractionStyle,

    /// Style for the tab body.
    pub tab_body: TabBodyStyle,

    /// If `true`, show the hline below the active tabs name.
    /// If `false`, show the active tab as merged with the tab ui area.
    /// By `Default` it's `false`.
    pub hline_below_active_tab_name: bool,

    /// The minimum width of the tab.
    ///
    /// The tab title or [`TabBarStyle::fill_tab_bar`] may make the tab
    /// wider than this but never shorter.
    pub minimum_width: Option<f32>,
}

/// Specifies the look and feel of individual tabs while they are being interacted with.
#[derive(Clone, Debug)]
pub struct TabInteractionStyle {
    /// Color of the outline around tabs. By `Default` it's [`Color32::BLACK`].
    pub outline_color: Color32,

    /// Tab rounding. By `Default` it's [`Rounding::default`]
    pub rounding: Rounding,

    /// Colour of the tab's background. By `Default` it's [`Color32::WHITE`]
    pub bg_fill: Color32,

    /// Color of the title text.
    pub text_color: Color32,
}

/// Specifies the look and feel of the tab body.
#[derive(Clone, Debug)]
pub struct TabBodyStyle {
    /// Inner margin of tab body. By `Default` it's `Margin::same(4.0)`
    pub inner_margin: Margin,

    /// The stroke of the tabs border. By `Default` it's ['Stroke::default']
    pub stroke: Stroke,

    /// Tab rounding. By `Default` it's [`Rounding::default`]
    pub rounding: Rounding,

    /// Colour of the tab's background. By `Default` it's [`Color32::WHITE`]
    pub bg_fill: Color32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            dock_area_padding: None,
            border: Stroke::new(f32::default(), Color32::BLACK),
            rounding: Rounding::default(),
            selection_color: Color32::from_rgb(0, 191, 255).linear_multiply(0.5),
            buttons: ButtonsStyle::default(),
            separator: SeparatorStyle::default(),
            tab_bar: TabBarStyle::default(),
            tab: TabStyle::default(),
            allowed_splits: SplitTypes::default(),
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
            add_tab_border_color: Color32::BLACK,
        }
    }
}

impl Default for SeparatorStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            extra_interact_width: 2.0,
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
            rounding: Rounding::default(),
            hline_color: Color32::BLACK,
            fill_tab_bar: false,
        }
    }
}

impl Default for TabStyle {
    fn default() -> Self {
        Self {
            active: TabInteractionStyle::default(),
            inactive: TabInteractionStyle {
                text_color: Color32::DARK_GRAY,
                ..Default::default()
            },
            focused: TabInteractionStyle {
                text_color: Color32::BLACK,
                ..Default::default()
            },
            hovered: TabInteractionStyle {
                text_color: Color32::BLACK,
                ..Default::default()
            },
            tab_body: TabBodyStyle::default(),
            hline_below_active_tab_name: false,
            minimum_width: None,
        }
    }
}

impl Default for TabInteractionStyle {
    fn default() -> Self {
        Self {
            bg_fill: Color32::WHITE,
            outline_color: Color32::BLACK,
            rounding: Rounding::default(),
            text_color: Color32::DARK_GRAY,
        }
    }
}

impl Default for TabBodyStyle {
    fn default() -> Self {
        Self {
            inner_margin: Margin::same(4.0),
            stroke: Stroke::default(),
            rounding: Rounding::default(),
            bg_fill: Color32::WHITE,
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
    ///
    /// See also: [`ButtonsStyle::from_egui`], [`SeparatorStyle::from_egui`], [`TabBarStyle::from_egui`],
    /// [`TabStyle::from_egui`]
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            border: Stroke::NONE,
            rounding: Rounding::none(),
            selection_color: style.visuals.selection.bg_fill.linear_multiply(0.5),
            buttons: ButtonsStyle::from_egui(style),
            separator: SeparatorStyle::from_egui(style),
            tab_bar: TabBarStyle::from_egui(style),
            tab: TabStyle::from_egui(style),
            ..Self::default()
        }
    }
}

impl ButtonsStyle {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`ButtonsStyle::close_tab_bg_fill`]
    /// - [`ButtonsStyle::close_tab_color`]
    /// - [`ButtonsStyle::close_tab_active_color`]
    /// - [`ButtonsStyle::add_tab_bg_fill`]
    /// - [`ButtonsStyle::add_tab_color`]
    /// - [`ButtonsStyle::add_tab_active_color`]
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            close_tab_bg_fill: style.visuals.widgets.hovered.bg_fill,
            close_tab_color: style.visuals.text_color(),
            close_tab_active_color: style.visuals.strong_text_color(),
            add_tab_bg_fill: style.visuals.widgets.hovered.bg_fill,
            add_tab_color: style.visuals.text_color(),
            add_tab_active_color: style.visuals.strong_text_color(),
            add_tab_border_color: style.visuals.widgets.noninteractive.bg_fill,
            ..ButtonsStyle::default()
        }
    }
}

impl SeparatorStyle {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`SeparatorStyle::color_idle`]
    /// - [`SeparatorStyle::color_hovered`]
    /// - [`SeparatorStyle::color_dragged`]
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            // Same as egui panel resize colors:
            color_idle: style.visuals.widgets.noninteractive.bg_stroke.color, // dim
            color_hovered: style.visuals.widgets.hovered.fg_stroke.color,     // bright
            color_dragged: style.visuals.widgets.active.fg_stroke.color,      // bright
            ..SeparatorStyle::default()
        }
    }
}

impl TabBarStyle {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`TabBarStyle::bg_fill`]
    /// - [`TabBarStyle::rounding`]
    /// - [`TabBarStyle::hline_color`]
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            bg_fill: style.visuals.extreme_bg_color,
            rounding: Rounding {
                nw: style.visuals.widgets.inactive.rounding.nw + 2.0,
                ne: style.visuals.widgets.inactive.rounding.ne + 2.0,
                sw: 0.0,
                se: 0.0,
            },
            hline_color: style.visuals.widgets.noninteractive.bg_stroke.color,
            ..TabBarStyle::default()
        }
    }
}

impl TabStyle {
    /// Derives tab styles from `egui::Style`.
    ///
    /// See also: [`TabInteractionStyle::from_egui_active`], [`TabInteractionStyle::from_egui_inactive`],
    /// [`TabInteractionStyle::from_egui_focused`], [`TabInteractionStyle::from_egui_hovered`], [`TabBodyStyle::from_egui`],
    pub fn from_egui(style: &egui::Style) -> TabStyle {
        Self {
            active: TabInteractionStyle::from_egui_active(style),
            inactive: TabInteractionStyle::from_egui_inactive(style),
            focused: TabInteractionStyle::from_egui_focused(style),
            hovered: TabInteractionStyle::from_egui_hovered(style),
            tab_body: TabBodyStyle::from_egui(style),
            ..Default::default()
        }
    }
}

impl TabInteractionStyle {
    /// Derives relevant fields from `egui::Style` for an active tab and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`TabInteractionStyle::outline_color`]
    /// - [`TabInteractionStyle::bg_fill`]
    /// - [`TabInteractionStyle::text_color`]
    /// - [`TabInteractionStyle::rounding`]
    pub fn from_egui_active(style: &egui::Style) -> Self {
        Self {
            outline_color: style.visuals.widgets.noninteractive.bg_stroke.color,
            bg_fill: style.visuals.window_fill(),
            text_color: style.visuals.text_color(),
            rounding: Rounding {
                sw: 0.0,
                se: 0.0,
                ..style.visuals.widgets.active.rounding
            },
        }
    }
    /// Derives relevant fields from `egui::Style` for an inactive tab and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`TabInteractionStyle::outline_color`]
    /// - [`TabInteractionStyle::bg_fill`]
    /// - [`TabInteractionStyle::text_color`]
    /// - [`TabInteractionStyle::rounding`]
    pub fn from_egui_inactive(style: &egui::Style) -> Self {
        Self {
            text_color: style.visuals.text_color(),
            bg_fill: egui::ecolor::tint_color_towards(
                style.visuals.window_fill,
                style.visuals.extreme_bg_color,
            ),
            outline_color: egui::ecolor::tint_color_towards(
                style.visuals.widgets.noninteractive.bg_stroke.color,
                style.visuals.extreme_bg_color,
            ),
            ..TabInteractionStyle::from_egui_active(style)
        }
    }
    /// Derives relevant fields from `egui::Style` for a focused tab and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`TabInteractionStyle::outline_color`]
    /// - [`TabInteractionStyle::bg_fill`]
    /// - [`TabInteractionStyle::text_color`]
    /// - [`TabInteractionStyle::rounding`]
    pub fn from_egui_focused(style: &egui::Style) -> Self {
        Self {
            text_color: style.visuals.strong_text_color(),
            ..TabInteractionStyle::from_egui_active(style)
        }
    }
    /// Derives relevant fields from `egui::Style` for a hovered tab and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`TabInteractionStyle::outline_color`]
    /// - [`TabInteractionStyle::bg_fill`]
    /// - [`TabInteractionStyle::text_color`]
    /// - [`TabInteractionStyle::rounding`]
    pub fn from_egui_hovered(style: &egui::Style) -> Self {
        Self {
            text_color: style.visuals.strong_text_color(),
            outline_color: style.visuals.widgets.hovered.bg_stroke.color,
            ..TabInteractionStyle::from_egui_inactive(style)
        }
    }
}

impl TabBodyStyle {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by [`egui::Style`] are:
    /// - [`TabBodyStyle::inner_margin`]
    /// - [`TabBodyStyle::stroke]
    /// - [`TabBodyStyle::rounding`]
    /// - [`TabBodyStyle::bg_fill`]
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            inner_margin: style.spacing.window_margin,
            stroke: style.visuals.widgets.noninteractive.bg_stroke,
            rounding: style.visuals.widgets.active.rounding,
            bg_fill: style.visuals.window_fill(),
        }
    }
}
