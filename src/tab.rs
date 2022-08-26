use egui::style::Margin;
use egui::{Frame, ScrollArea, Ui, WidgetText};

pub type TabContent = Box<dyn FnMut(&mut Ui) + 'static>;
pub type OnClose = Box<dyn FnMut() -> bool + 'static>;
pub type ForceClose = Box<dyn FnMut() -> bool + 'static>;

pub struct TabBuilder {
    title: Option<WidgetText>,
    inner_margin: Margin,
    add_content: Option<TabContent>,
    on_close: Option<OnClose>,
    force_close: Option<ForceClose>,
}

/// Dockable tab that can be used in `Tree`s.
pub trait Tab {
    /// Actual tab content.
    fn ui(&mut self, ui: &mut Ui);

    /// The title to be displayed.
    fn title(&mut self) -> WidgetText;

    /// This is called when the tabs close button is pressed.
    ///
    /// Returns `true` if the tab should close immediately, `false` otherwise.
    ///
    /// NOTE if returning false `ui` will still be called once more if this tab is active.
    fn on_close(&mut self) -> bool {
        true
    }

    /// This is called every frame after `ui` is called (if the tab is active).
    ///
    /// Returns `true` if the tab should be forced to close, `false` otherwise.
    ///
    /// In the event this function returns true the tab will be removed without calling `on_close`.
    fn force_close(&mut self) -> bool {
        false
    }
}

pub struct BuiltTab {
    pub title: WidgetText,
    pub inner_margin: Margin,
    pub add_content: TabContent,
    on_close: Option<OnClose>,
    force_close: Option<ForceClose>,
}

impl Tab for BuiltTab {
    fn ui(&mut self, ui: &mut Ui) {
        ScrollArea::both()
            .id_source(self.title.text().to_string() + " - egui_dock::Tab")
            .show(ui, |ui| {
                Frame::none()
                    .inner_margin(self.inner_margin)
                    .show(ui, |ui| {
                        let available_rect = ui.available_rect_before_wrap();
                        ui.expand_to_include_rect(available_rect);
                        (self.add_content)(ui);
                    });
            });
    }

    fn title(&mut self) -> WidgetText {
        self.title.clone()
    }

    fn on_close(&mut self) -> bool {
        match &mut self.on_close {
            Some(on_close) => on_close(),
            None => true,
        }
    }

    fn force_close(&mut self) -> bool {
        match &mut self.force_close {
            Some(force_close) => force_close(),
            None => false,
        }
    }
}

impl Default for TabBuilder {
    fn default() -> Self {
        Self {
            title: None,
            inner_margin: Margin::same(4.0),
            add_content: None,
            on_close: None,
            force_close: None,
        }
    }
}

impl TabBuilder {
    /// Constructs a `Tab` out of accumulated data.
    ///
    /// # Panics
    /// Panics if `title` or `add_contents` is unset.
    pub fn build(self) -> Box<dyn Tab> {
        Box::new(BuiltTab {
            title: self.title.expect("Missing tab title"),
            inner_margin: self.inner_margin,
            add_content: self.add_content.expect("Missing tab content"),
            on_close: self.on_close,
            force_close: self.force_close,
        })
    }

    /// Sets the text displayed in the tab bar.
    pub fn title(mut self, title: impl Into<WidgetText>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the margins around the tab's content.
    pub fn inner_margin(mut self, margin: Margin) -> Self {
        self.inner_margin = margin;
        self
    }

    /// Sets the function that adds content to the tab.
    pub fn content(mut self, add_content: impl FnMut(&mut Ui) + 'static) -> Self {
        self.add_content = Some(Box::new(add_content));
        self
    }

    /// Sets the function that is called when the close button is pressed.
    /// The function should return `true` if the tab should close immediately, `false` otherwise.
    ///
    /// If no function is set the default behavior is to always return true.
    ///
    /// See [Tab](crate::tab::Tab) `on_close` for more detail
    pub fn on_close(mut self, on_close: impl FnMut() -> bool + 'static) -> Self {
        self.on_close = Some(Box::new(on_close));
        self
    }

    /// Sets the function that is called every frame to determine if the tab should close.
    /// The function should return `true` if the tab should be forced to close, `false` otherwise.
    ///
    /// If no function is set the default behavior is to always return false.
    ///
    /// See [Tab](crate::tab::Tab) `force_close` for more detail
    pub fn force_close(mut self, force_close: impl FnMut() -> bool + 'static) -> Self {
        self.force_close = Some(Box::new(force_close));
        self
    }
}
