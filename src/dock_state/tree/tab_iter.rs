use crate::{NodeIndex, Tree};

/// Iterates over all tabs in a [`Tree`].
pub struct TabIter<'a, Tab> {
    tree: &'a Tree<Tab>,
    node_order: Vec<NodeIndex>,
    node_position: usize,
    tab_idx: usize,
}

impl<'a, Tab> TabIter<'a, Tab> {
    pub(super) fn new(tree: &'a Tree<Tab>) -> Self {
        Self {
            tree,
            node_order: tree.breadth_first_index_iter().collect(),
            node_position: 0,
            tab_idx: 0,
        }
    }
}

impl<'a, Tab> Iterator for TabIter<'a, Tab> {
    type Item = &'a Tab;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node_idx = self.node_order.get(self.node_position)?;
            match self.tree[*node_idx]
                .tabs()
                .and_then(|tabs| tabs.get(self.tab_idx))
            {
                Some(tab) => {
                    self.tab_idx += 1;
                    return Some(tab);
                }
                None => {
                    self.node_position += 1;
                    self.tab_idx = 0;
                }
            }
        }
    }
}

impl<'a, Tab> std::fmt::Debug for TabIter<'a, Tab> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TabIter").finish_non_exhaustive()
    }
}

#[test]
fn test_tabs_iter() {
    fn tabs(tree: &Tree<i32>) -> Vec<i32> {
        tree.tabs().copied().collect()
    }

    let mut tree = Tree::new(vec![1, 2, 3]);
    assert_eq!(tabs(&tree), vec![1, 2, 3]);

    tree.push_to_first_leaf(4);
    assert_eq!(tabs(&tree), vec![1, 2, 3, 4]);

    tree.push_to_first_leaf(5);
    assert_eq!(tabs(&tree), vec![1, 2, 3, 4, 5]);

    tree.push_to_focused_leaf(6);
    assert_eq!(tabs(&tree), vec![1, 2, 3, 4, 5, 6]);

    assert_eq!(tree.num_tabs(), tree.tabs().count());
}
