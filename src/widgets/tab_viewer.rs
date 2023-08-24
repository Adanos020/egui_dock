use crate::{NodeIndex, TabStyle};
use egui::{Id, Ui, WidgetText};

/// Defines how to display a tab inside a [`Tree`](crate::Tree).
pub trait TabViewer {
    /// The type of tab in which you can store state to be drawn in your tabs.
    type Tab;

    /// The title to be displayed in the tab bar.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText;

    /// Actual tab content.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab);

    /// Content inside the context menu shown when the tab is right-clicked.
    ///
    /// The `_node` specifies which [`Node`](crate::Node) or split of the tree that this particular context menu
    /// belongs to.
    fn context_menu(&mut self, _ui: &mut Ui, _tab: &mut Self::Tab, _node: NodeIndex) {}

    /// Unique ID for this tab.
    ///
    /// If not implemented, uses tab title text as an id source.
    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(self.title(tab).text())
    }

    /// Called after each tab button is shown, so you can add a tooltip, check for clicks, etc.
    fn on_tab_button(&mut self, _tab: &mut Self::Tab, _response: &egui::Response) {}

    /// Called before showing the close button.
    ///
    /// Return `false` if the close buttons should not be shown.
    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    /// This is called when the tabs close button is pressed.
    ///
    /// Returns `true` if the tab should close immediately, otherwise `false`.
    ///
    /// **Note**: if `false` is returned, [`ui`](Self::ui) will still be called once more if this tab is active.
    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    /// This is called when the tabs add button is pressed and if the dock [`Style`](crate::Style)'s
    /// `show_add_buttons` is set to `true`.
    ///
    /// `_node` specifies which [`Node`](crate::Node) or split of the tree this particular add button was pressed on.
    fn on_add(&mut self, _node: NodeIndex) {}

    /// Content of the popup under the add button. Useful for selecting what type of tab to add.
    ///
    /// This requires that [`DockArea::show_add_buttons`](crate::DockArea::show_add_buttons) and
    /// [`DockArea::show_add_popup`](crate::DockArea::show_add_popup) are set to `true`.
    fn add_popup(&mut self, _ui: &mut Ui, _node: NodeIndex) {}

    /// This is called every frame after [`ui`](Self::ui) is called, if the `_tab` is active.
    ///
    /// Returns `true` if the tab should be forced to close, `false` otherwise.
    ///
    /// In the event this function returns true the tab will be removed without calling `on_close`.
    fn force_close(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    /// Sets custom style for given tab.
    fn tab_style_override(&self, _tab: &Self::Tab, _global_style: &TabStyle) -> Option<TabStyle> {
        None
    }

    /// Specifies a tab's ability to be shown in a window.
    ///
    /// Return `false` if you don't want this tab to be turned into a window.
    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        true
    }

    /// Whether the tab will be cleared with the color specified in [`TabBarStyle::bg_fill`](crate::TabBarStyle::bg_fill).
    fn clear_background(&self, _tab: &Self::Tab) -> bool {
        true
    }
    /// The scrolling of each tab is controlled by themselves

    fn tab_scroll(&self, _tab: &Self::Tab) -> bool {
        false
    }
}
