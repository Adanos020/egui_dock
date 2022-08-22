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
pub trait Tab{
    ///Actual tab content
    fn ui(&mut self, ui: &mut egui::Ui);
    ///The title to be displayed
    fn title(&mut self) -> egui::WidgetText;
    /// This is called when the close button is pressed
    /// returning false will cancel closing the tab
    fn on_close(&mut self) -> bool{
        true
    }
    /// This is called every frame
    /// return true and the tab will close 
    /// using this to close the tab will NOT call the on_close function!
    fn force_close(&mut self) -> bool{
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
impl Tab for BuiltTab{
    fn title(&mut self) -> egui::WidgetText {
        self.title.clone()
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
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

    fn on_close(&mut self) -> bool {
        match &mut self.on_close{
            Some(on_close) => on_close(),
            None => true,
        }
    }

    fn force_close(&mut self) -> bool {
        match &mut self.force_close{
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
    
    /// Sets the function that runs when the close button is pressed
    /// return true to close and false to block the close
    pub fn on_close(mut self, on_close: impl FnMut() -> bool + 'static) -> Self {
        self.on_close = Some(Box::new(on_close));
        self
    }
    
    /// Sets the function that checks if the tab should be closed every frame
    /// return false to keep open and true to close the tab
    /// returning true will NOT call the on_close function (if any)
    pub fn force_close(mut self, force_close: impl FnMut() -> bool + 'static) -> Self {
        self.force_close = Some(Box::new(force_close));
        self
    }
}

