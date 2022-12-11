# `egui_dock`: docking support for [egui](https://github.com/emilk/egui)

[![egui_ver](https://img.shields.io/badge/egui-0.20-blue)](https://github.com/emilk/egui)
[![Crates.io](https://img.shields.io/crates/v/egui_dock)](https://crates.io/crates/egui_dock)
[![docs.rs](https://img.shields.io/docsrs/egui_dock)](https://docs.rs/egui_dock/)

Originally created by [@lain-dono](https://github.com/lain-dono), this library provides docking support for `egui`.
It lets you open and close tabs, freely move them around, insert them in selected parts of the `DockArea`, and resize them.

## How to contribute

Feel free to open new issues and pull requests.

Before contributing, please read [the contribution guide](CONTRIBUTING.md).

## Quick start

Add `egui_dock` to your project's dependencies.

```toml
[dependencies]
egui_dock = "0.3"
```

Instead of explicitly depending on `egui`, prefer using `egui_dock::egui` since it's the compatible version.

Then proceed by setting up `egui`, following its [quick start guide](https://github.com/emilk/egui#quick-start).
Once that's done, you can start using `egui_dock` – more details on that can be found in the
[documentation](https://docs.rs/egui_dock/latest/egui_dock/).

## Demo

![demo](images/demo.gif "Demo")
