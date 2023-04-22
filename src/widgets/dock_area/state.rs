use egui::{Context, Id, Pos2};

#[derive(Clone, Debug, Default)]
pub(super) struct State {
    pub drag_start: Option<Pos2>,
}

impl State {
    #[inline(always)]
    pub(super) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|d| d.get_temp(id))
            .unwrap_or(Self { drag_start: None })
    }

    #[inline(always)]
    pub(super) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }
}
