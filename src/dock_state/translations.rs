/// Groups together labels from different elements of the [`DockArea`](crate::DockArea).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Translations {
    /// Text overrides for buttons in tab context menus.
    pub tab_context_menu: TabContextMenuTranslations,
    /// Text overrides for buttons in windows.
    pub leaf: LeafTranslations,
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

/// Specifies text displayed in the primary buttons on a tab bar.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LeafTranslations {
    /// Message in the tooltip shown while hovering over a grayed out X button of a leaf
    /// containing non-closable tabs.
    pub close_button_tooltip: String,
    /// Button that closes the entire window.
    pub close_all_button: String,
    /// Message in the tooltip shown while hovering over a grayed out close window button of a window
    /// containing non-closable tabs.
    pub close_all_button_tooltip: String,
    /// Button that minimizes the window.
    pub minimize_button: String,
}

impl Translations {
    /// Default English translations.
    pub fn english() -> Self {
        Self {
            tab_context_menu: TabContextMenuTranslations::english(),
            leaf: LeafTranslations::english(),
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

impl LeafTranslations {
    /// Default English translations.
    pub fn english() -> Self {
        Self {
            close_button_tooltip: String::from("This leaf contains non-closable tabs."),
            close_all_button: String::from("Close window"),
            close_all_button_tooltip: String::from("This window contains non-closable tabs."),
            minimize_button: String::from("Minimize window"),
        }
    }
}
