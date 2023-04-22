use crate::{NodeIndex, Split, TabDestination, TabIndex};
use egui::{Pos2, Rect};

#[derive(Debug)]
pub(super) struct HoverData {
    pub rect: Rect,
    pub tabs: Option<Rect>,
    pub tab: Option<(Rect, TabIndex)>,
    pub dst: NodeIndex,
    pub pointer: Pos2,
}

impl HoverData {
    pub(super) fn resolve(&self) -> (Rect, TabDestination) {
        if let Some(tab) = self.tab {
            return (tab.0, TabDestination::Insert(tab.1));
        }
        if let Some(tabs) = self.tabs {
            return (tabs, TabDestination::Append);
        }

        let (rect, pointer) = (self.rect, self.pointer);

        let center = rect.center();
        let pts = [
            (
                center.distance(pointer),
                TabDestination::Append,
                Rect::EVERYTHING,
            ),
            (
                rect.left_center().distance(pointer),
                TabDestination::Split(Split::Left),
                Rect::everything_left_of(center.x),
            ),
            (
                rect.right_center().distance(pointer),
                TabDestination::Split(Split::Right),
                Rect::everything_right_of(center.x),
            ),
            (
                rect.center_top().distance(pointer),
                TabDestination::Split(Split::Above),
                Rect::everything_above(center.y),
            ),
            (
                rect.center_bottom().distance(pointer),
                TabDestination::Split(Split::Below),
                Rect::everything_below(center.y),
            ),
        ];

        let (_, tab_dst, overlay) = pts
            .into_iter()
            .min_by(|(lhs, ..), (rhs, ..)| lhs.total_cmp(rhs))
            .unwrap();

        (rect.intersect(overlay), tab_dst)
    }
}
