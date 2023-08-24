use crate::{NodeIndex, TabStyle};
use egui::{Id, Ui, WidgetText};

/// Defines how to display a tab inside a [`Tree`](crate::Tree).
pub trait TabViewer {
    /// The type of tab in which you can store state to be drawn in your tabs.
    type Tab;

    /// Actual tab content.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab);

    /// Content inside context_menu.
    fn context_menu(&mut self, _ui: &mut Ui, _tab: &mut Self::Tab) {}

    /// The title to be displayed.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText;

    /// Unique id for this tab.
    ///
    /// If not implemented, uses tab title text as an id source.
    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(self.title(tab).text())
    }

    /// Called after each tab button is shown, so you can add a tooltip, check for clicks, etc.
    fn on_tab_button(&mut self, _tab: &mut Self::Tab, _response: &egui::Response) {}

    /// This is called when the tabs close button is pressed.
    ///
    /// Returns `true` if the tab should close immediately, `false` otherwise.
    ///
    /// NOTE if returning false `ui` will still be called once more if this tab is active.
    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    /// This is called when the tabs add button is pressed.
    ///
    /// This requires the dock style's `show_add_buttons` to be `true`.
    ///
    /// The `_node` specifies which `Node` or split of the tree that this
    /// particular add button was pressed on.
    fn on_add(&mut self, _node: NodeIndex) {}

    /// Content of add_popup. Displays a popup under the add button. Useful for selecting
    /// what type of tab to add.
    ///
    /// This requires the dock style's `show_add_buttons` and `show_add_popup` to be `true`.
    fn add_popup(&mut self, _ui: &mut Ui, _node: NodeIndex) {}

    /// This is called every frame after `ui` is called (if the tab is active).
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

    /// Whether the tab will be cleared with the color specified in [`TabBarStyle::bg_fill`](crate::TabBarStyle::bg_fill)
    fn clear_background(&self, _tab: &Self::Tab) -> bool {
        true
    }

    /// allows setting if individual tabs should show the close button. 
    /// only works if show_close_buttons is also set.
    /// will defaul to true if not implemented. 
    fn show_close_button(&self, _tab: &Self::Tab) -> bool {
        true
    }

    /// allow setting if an individaul tabs should be draggable
    /// only workd if draggable_tabs is set to true
    /// will default to true if not implemented
    fn allow_drag(&self, _tab: &Self::Tab) -> bool {
        true
    }

    /// allow setting if an individual tab should show the title bar
    /// will default to true if not implemented. 
    fn show_title(&self, _tab: &Self::Tab) -> bool {
        true
    }
}
