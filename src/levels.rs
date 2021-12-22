use crate::graph_types::{StrandNode, Strand};


pub(crate) struct Level
{
    pub start: Strand,
    pub target: Strand,
}

pub(crate) fn makeSequence1Strand1() -> Level
{
    let mut start = Strand::default();
    for _ in 0..3 {
        start.add_node(StrandNode {});
    }
    start.extend_with_edges(&[(0,1), (1,2)]);

    let mut target = Strand::default();
    for _ in 0..3 {
        target.add_node(StrandNode {});
    }
    target.extend_with_edges(&[(0,1), (0, 2)]);

    Level{start, target}
}

pub(crate) fn makeSequence1Strand2() -> Level
{
    let mut start = Strand::default();
    for _ in 0..5 {
        start.add_node(StrandNode {});
    }
    start.extend_with_edges(&[(0,1), (1,2), (2,3), (1,4)]);

    let mut target = Strand::default();
    for _ in 0..5 {
        target.add_node(StrandNode {});
    }
    target.extend_with_edges(&[(0,1), (1,2), (0,3), (3,4)]);

    Level{start, target}
}
