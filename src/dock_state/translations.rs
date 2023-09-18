/// Groups together labels from different elements of the [`DockArea`](crate::DockArea).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Translations {
    /// Text overrides for buttons in tab context menus.
    pub tab_context_menu: TabContextMenuTranslations,
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

impl Default for Translations {
    /// Default English translations.
    fn default() -> Self {
        Self {
            tab_context_menu: TabContextMenuTranslations::default(),
        }
    }
}

impl Default for TabContextMenuTranslations {
    /// Default English translations.
    fn default() -> Self {
        Self {
            close_button: String::from("Close"),
            eject_button: String::from("Eject"),
        }
    }
}
