/// Groups together labels from different elements of the [`DockArea`](crate::DockArea).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Translations {
    /// Text overrides for buttons in tab context menus.
    pub tab_context_menu: TabContextMenuTranslations,
    /// Text overrides for buttons in windows.
    pub window: WindowTranslations,
}

/// Specifies text in buttons displayed in the context menu displayed upon right-clicking on a tab.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TabContextMenuTranslations {
    /// Button that closes the tab.
    pub close_button: String,
    /// Button that undocks the tab into a new window.
    pub eject_button: String,
}

/// Specifies text in buttons displayed in the context menu displayed upon right-clicking on a tab.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WindowTranslations {
    /// Message in the tooltip shown while hovering over a grayed out X button of a window
    /// containing non-closable tabs.
    pub close_button_tooltip: String,
}

impl Translations {
    /// Default English translations.
    pub fn english() -> Self {
        Self {
            tab_context_menu: TabContextMenuTranslations::english(),
            window: WindowTranslations::english(),
        }
    }
}

impl TabContextMenuTranslations {
    /// Default English translations.
    pub fn english() -> Self {
        Self {
            close_button: String::from("Close"),
            eject_button: String::from("Eject"),
        }
    }
}

impl WindowTranslations {
    /// Default English translations.
    pub fn english() -> Self {
        Self {
            close_button_tooltip: String::from("This window contains non-closable tabs."),
        }
    }
}
