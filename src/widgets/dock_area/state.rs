use std::time::Instant;

use egui::{Context, Id, Pos2};

use crate::{Style, SurfaceIndex};

use super::hover_data::{DragData, DragDropState, HoverData};

#[derive(Clone, Debug, Default)]
pub(super) struct State {
    pub drag_start: Option<Pos2>,
    pub drag: Option<DragDropState>,
    pub window_fade: Option<(Instant, SurfaceIndex)>,
}

impl State {
    #[inline(always)]
    pub(super) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|d| d.get_temp(id)).unwrap_or(Self {
            drag_start: None,
            drag: None,
            window_fade: None,
        })
    }

    #[inline(always)]
    pub(super) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }

    #[inline(always)]
    pub(super) fn is_drag_drop_lock_some(&self) -> bool {
        self.drag.as_ref().and_then(|drag| drag.locked).is_some()
    }

    pub(super) fn reset_drag(&mut self) {
        self.drag = None;
        self.window_fade = None;
    }

    // HACKY: Fix asap!
    pub(super) fn set_drag_and_drop(
        &mut self,
        drag: DragData,
        drop: HoverData,
        ctx: &Context,
        style: &Style,
    ) {
        if !self.is_drag_drop_locked(ctx, style) {
            self.drag = Some(DragDropState {
                hover: drop,
                drag,
                pointer: ctx.pointer_hover_pos().unwrap_or(Pos2::ZERO),
                locked: None,
            })
        }
    }

    #[inline(always)]
    fn is_drag_drop_locked(&self, ctx: &Context, style: &Style) -> bool {
        self.drag
            .as_ref()
            .is_some_and(|drag_drop_state| drag_drop_state.is_locked(style, ctx))
    }
}
