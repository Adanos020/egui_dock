# `egui_dock`: docking support for `egui`

Credit goes to [@Iain-dono](https://github.com/lain-dono) for implementing the actual library.

This fork aims to provide documentation and further development if necessary.

## Demo

![demo](images/demo.gif "Demo")

## Usage

First, create your context type and your tab widget:

```rust
use egui::{Frame, Ui, style::Margin};
use egui_dock::Tab;

struct MyContext;

struct MyTab {
    text: String,
}

impl MyTab {
    fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl Tab<MyContext> for MyTab {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut Ui, _ctx: &mut MyContext) {
        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            ui.label(&self.text);
        });
    }
}
```

Then construct the initial tree using your tab widget:

```rust
use egui_dock::{NodeIndex, Tree};

let tab1 = Box::new(MyTab::new("Tab 1"));
let tab2 = Box::new(MyTab::new("Tab 2"));
let tab3 = Box::new(MyTab::new("Tab 3"));
let tab4 = Box::new(MyTab::new("Tab 4"));
let tab5 = Box::new(MyTab::new("Tab 5"));

let mut tree = Tree::new(vec![tab1, tab2]);

// You can modify the tree in runtime
let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![tab3]);
let [_, _] = tree.split_below(a, 0.7, vec![tab4]);
let [_, _] = tree.split_below(b, 0.5, vec![tab5]);
```

Finally, you can show the tree.

```rust
let id = ui.id();
egui_dock::show(&mut ui, id, style, tree, context);
```

## Contribution

Feel free to open issues and pull requests.
