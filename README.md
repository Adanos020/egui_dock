# `egui_dock`: docking support for [egui](https://github.com/emilk/egui)

[![egui_ver](https://img.shields.io/badge/egui-0.19-blue)](https://github.com/emilk/egui)
[![Crates.io](https://img.shields.io/crates/v/egui_dock)](https://crates.io/crates/egui_dock)
[![docs.rs](https://img.shields.io/docsrs/egui_dock)](https://docs.rs/egui_dock/)

Credit goes to [@lain-dono](https://github.com/lain-dono) for implementing the actual library.

This fork aims to provide documentation and further development if necessary.

## How to contribute

Before contributing, please read [the contribution guide](CONTRIBUTING.md).

## Quick start

Add `egui_dock` to your project's dependencies.

```toml
[dependencies]
egui_dock = "0.2"
```

Instead of explicitly depending on `egui`, prefer using `egui_dock_::egui` since it is the compatible version.

Then proceed by setting up `egui`, following its [quick start guide](https://github.com/emilk/egui#quick-start).
More details on the usage of `egui_dock` can be found in the [documentation](https://docs.rs/egui_dock/latest/egui_dock/).

## Demo

![demo](images/demo.gif "Demo")
