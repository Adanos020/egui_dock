use std::time::Instant;

use egui::{Context, Id, Pos2};

use crate::SurfaceIndex;

use super::hover_data::HoverData;

#[derive(Clone, Debug, Default)]
pub(super) struct State {
    pub drag_start: Option<Pos2>,
    pub hover_data: Option<HoverData>,
    pub window_fade: Option<(Instant, SurfaceIndex)>,
}

impl State {
    #[inline(always)]
    pub(super) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|d| d.get_temp(id)).unwrap_or(Self {
            drag_start: None,
            hover_data: None,
            window_fade: None,
        })
    }

    #[inline(always)]
    pub(super) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }
}
