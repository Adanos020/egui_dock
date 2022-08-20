use egui::style::Margin;
use egui::{Frame, ScrollArea, Ui, WidgetText};

pub type TabContent = Box<dyn FnMut(&mut Ui) + 'static>;

pub struct TabBuilder {
    title: Option<WidgetText>,
    inner_margin: Margin,
    add_content: Option<TabContent>,
}

/// Dockable tab that can be used in `Tree`s.
pub struct Tab {
    pub title: WidgetText,
    pub inner_margin: Margin,
    pub add_content: TabContent,
}

impl Default for TabBuilder {
    fn default() -> Self {
        Self {
            title: None,
            inner_margin: Margin::same(4.0),
            add_content: None,
        }
    }
}

impl TabBuilder {
    /// Constructs a `Tab` out of accumulated data.
    ///
    /// # Panics
    /// Panics if `title` or `add_contents` is unset.
    pub fn build(self) -> Tab {
        Tab {
            title: self.title.expect("Missing tab title"),
            inner_margin: self.inner_margin,
            add_content: self.add_content.expect("Missing tab content"),
        }
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
}

impl Tab {
    /// Displays the tab's content.
    pub fn ui(&mut self, ui: &mut Ui) {
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
}
