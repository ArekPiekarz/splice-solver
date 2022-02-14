use crate::strand::{CellKind, Edge, NodeId, Strand};

use anyhow::{bail, Result};


pub(crate) fn makeLevel(sequence: SequenceNumber, strand: StrandNumber) -> Result<Level>
{
    if sequence.0 == 0 {
        bail!("Sequence number must start at 1, got 0.")
    }
    if strand.0 == 0 {
        bail!("Strand number must start at 1, got 0.")
    }

    match sequence.0 {
        1 => makeStrandInSequence1(strand),
        2 => makeStrandInSequence2(strand),
        3 => makeStrandInSequence3(strand),
        4 => makeStrandInSequence4(strand),
        _ => bail!("Unsupported sequence number: {}", sequence.0)
    }
}

fn makeStrandInSequence1(strand: StrandNumber) -> Result<Level>
{
    let levelInfo = match strand.0 {
        1 => LevelInfo{
            start:  StrandInfo{nodeCount: 3, edges: vec![(0,1), (1,2)], mutables: vec![]},
            target: StrandInfo{nodeCount: 3, edges: vec![(0,1), (0,2)], mutables: vec![]},
            maxSplices: 1},
        2 => LevelInfo{
            start:  StrandInfo{nodeCount: 5, edges: vec![(0,1), (1,2), (2,3), (1,4)], mutables: vec![]},
            target: StrandInfo{nodeCount: 5, edges: vec![(0,1), (1,2), (0,3), (3,4)], mutables: vec![]},
            maxSplices: 1}, // really 2, but it's angelic
        3 => LevelInfo{
            start:  StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (1,3), (3,4), (3,5), (0,6), (6,7), (6,8), (8,9), (8,10)], mutables: vec![]},
            target: StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (2,3), (3,4), (3,5), (0,6), (6,7), (7,8), (8,9), (8,10)], mutables: vec![]},
            maxSplices: 2},
        4 => LevelInfo{
            start:  StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (2,3), (2,4), (4,5), (1,6), (6,7), (7,8), (6,9), (0,10)], mutables: vec![]},
            target: StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (2,3), (3,4), (2,5), (0,6), (6,7), (7,8), (7,9), (9,10)], mutables: vec![]},
            maxSplices: 1},
        5 => LevelInfo{
            start:  StrandInfo{nodeCount: 10, edges: vec![(0,1), (1,2), (1,3), (0,4), (4,5), (4,6), (6,7), (7,8), (7,9)], mutables: vec![]},
            target: StrandInfo{nodeCount: 10, edges: vec![(0,1), (1,2), (2,3), (3,4), (3,5), (1,6), (6,7), (7,8), (7,9)], mutables: vec![]},
            maxSplices: 1}, // really 2, but it's angelic
        6 => LevelInfo{
            start:  StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (1,3), (3,4), (4,5), (4,6), (0,7), (7,8), (8,9), (9,10), (9,11),  (7,12)], mutables: vec![]},
            target: StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (2,3), (3,4), (3,5), (1,6), (0,7), (7,8), (7,9), (9,10), (10,11), (10,12)], mutables: vec![]},
            maxSplices: 1}, // really 2, but it's angelic
        7 => LevelInfo{
            start:  StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (2,3), (3,4), (3,5), (5,6), (5,7), (2,8), (8,9), (9,10), (9,11), (8,12)], mutables: vec![]},
            target: StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (2,3), (2,4), (4,5), (4,6), (0,7), (7,8), (8,9), (9,10), (9,11), (8,12)], mutables: vec![]},
            maxSplices: 2},
        _ => bail!("Unsupported strand number: {}", strand.0)
    };
    Ok(makeLevelFrom(levelInfo))
}

fn makeStrandInSequence2(strand: StrandNumber) -> Result<Level>
{
    use CellKind::Doubler;
    let levelInfo = match strand.0 {
        1 => LevelInfo{
            start:  StrandInfo{nodeCount: 3, edges: vec![(0,1), (1,2)], mutables: vec![(1, Doubler)]},
            target: StrandInfo{nodeCount: 5, edges: vec![(0,1), (1,2), (0,3), (3,4)], mutables: vec![]},
            maxSplices: 1},
        2 => LevelInfo{
            start:  StrandInfo{nodeCount: 4, edges: vec![(0,1), (0,2), (2,3)], mutables: vec![(3, Doubler)]},
            target: StrandInfo{nodeCount: 6, edges: vec![(0,1), (1,2), (2,3), (1,4), (4,5)], mutables: vec![]},
            maxSplices: 1},
        3 => LevelInfo{
            start:  StrandInfo{nodeCount: 5, edges: vec![(0,1), (1,2), (1,3), (3,4)], mutables: vec![(4, Doubler)]},
            target: StrandInfo{nodeCount: 7, edges: vec![(0,1), (1,2), (2,3), (0,4), (4,5), (5,6)], mutables: vec![]},
            maxSplices: 3},
        4 => LevelInfo{
            start:  StrandInfo{nodeCount: 8,  edges: vec![(0,1), (1,2), (2,3), (2,4), (4,5), (1,6), (6,7)], mutables: vec![(1, Doubler)]},
            target: StrandInfo{nodeCount: 15, edges: vec![(0,1), (1,2), (2,3), (2,4), (1,5), (5,6), (5,7), (0,8), (8,9), (9,10), (9,11), (8,12), (12,13), (12,14)], mutables: vec![]},
            maxSplices: 1}, // really 2, but it's angelic
        5 => LevelInfo{
            start:  StrandInfo{nodeCount: 5, edges: vec![(0,1), (1,2), (2,3), (1,4)], mutables: vec![(3, Doubler)]},
            target: StrandInfo{nodeCount: 8, edges: vec![(0,1), (1,2), (2,3), (3,4), (1,5), (5,6), (6,7)], mutables: vec![]},
            maxSplices: 2}, // really 3, but it's angelic
        6 => LevelInfo{
            start:  StrandInfo{nodeCount: 8,  edges: vec![(0,1), (1,2), (2,3), (3,4), (3,5), (1,6), (6,7)], mutables: vec![(3, Doubler)]},
            target: StrandInfo{nodeCount: 12, edges: vec![(0,1), (1,2), (2,3), (3,4), (4,5), (3,6), (1,7), (7,8), (8,9), (8,10), (10,11)], mutables: vec![]},
            maxSplices: 3},
        7 => LevelInfo{
            start:  StrandInfo{nodeCount: 3, edges: vec![(0,1), (1,2)], mutables: vec![(1, Doubler), (2, Doubler)]},
            target: StrandInfo{nodeCount: 9, edges: vec![(0,1), (1,2), (2,3), (2,4), (0,5), (5,6), (6,7), (6,8)], mutables: vec![]},
            maxSplices: 2},
        _ => bail!("Unsupported strand number: {}", strand.0)
    };
    Ok(makeLevelFrom(levelInfo))
}

fn makeStrandInSequence3(strand: StrandNumber) -> Result<Level>
{
    use CellKind::{Doubler, Extender};
    let levelInfo = match strand.0 {
        1 => LevelInfo{
            start:  StrandInfo{nodeCount: 4, edges: vec![(0,1), (1,2), (1,3)], mutables: vec![(1, Doubler), (3, Extender)]},
            target: StrandInfo{nodeCount: 9, edges: vec![(0,1), (1,2), (1,3), (3,4), (0,5), (5,6), (6,7), (5,8)], mutables: vec![]},
            maxSplices: 1},
        2 => LevelInfo{
            start:  StrandInfo{nodeCount: 6, edges: vec![(0,1), (1,2), (1,3), (3,4), (4,5)], mutables: vec![(1, Extender), (4, Extender), (5, Extender)]},
            target: StrandInfo{nodeCount: 9, edges: vec![(0,1), (1,2), (2,3), (2,4), (0,5), (5,6), (6,7), (6,8)], mutables: vec![]},
            maxSplices: 2}, // really 3, but it's angelic
        3 => LevelInfo{
            start:  StrandInfo{nodeCount: 5,  edges: vec![(0,1), (1,2), (2,3), (3,4)], mutables: vec![(1, Doubler), (4, Extender)]},
            target: StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (2,3), (1,4), (4,5), (0,6), (6,7), (7,8), (6,9), (9,10)], mutables: vec![]},
            maxSplices: 1}, // really 2, but it's angelic
        4 => LevelInfo{
            start:  StrandInfo{nodeCount: 7,  edges: vec![(0,1), (1,2), (1,3), (0,4), (4,5), (5,6)], mutables: vec![(4, Extender), (5, Doubler)]},
            target: StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (2,3), (3,4), (4,5), (3,6), (6,7), (2, 8), (8,9), (9,10), (8,11), (11,12)], mutables: vec![]},
            maxSplices: 2}, // really 3, but it's angelic
        5 => LevelInfo{
            start:  StrandInfo{nodeCount: 7,  edges: vec![(0,1), (0,2), (2,3), (3,4), (2,5), (5,6)], mutables: vec![(1, Extender), (4, Doubler), (6, Doubler)]},
            target: StrandInfo{nodeCount: 10, edges: vec![(0,1), (1,2), (2,3), (2,4), (4,5), (1,6), (6,7), (7,8), (6,9)], mutables: vec![]},
            maxSplices: 2},
        6 => LevelInfo{
            start:  StrandInfo{nodeCount: 6, edges: vec![(0,1), (1,2), (1,3), (3,4), (4,5)], mutables: vec![(3, Extender), (4, Doubler), (5, Doubler)]},
            target: StrandInfo{nodeCount: 9, edges: vec![(0,1), (1,2), (2,3), (2,4), (0,5), (5,6), (6,7), (6,8)], mutables: vec![]},
            maxSplices: 2},
        7 => LevelInfo{
            start:  StrandInfo{nodeCount: 6,  edges: vec![(0,1), (0,2), (2,3), (3,4), (4,5)], mutables: vec![(3, Doubler), (4, Doubler), (5, Extender)]},
            target: StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (2,3), (1,4), (4,5), (0,6), (6,7), (7,8), (7,9), (6,10), (10,11), (10,12)], mutables: vec![]},
            maxSplices: 2},
        _ => bail!("Unsupported strand number: {}", strand.0)
    };
    Ok(makeLevelFrom(levelInfo))
}

fn makeStrandInSequence4(strand: StrandNumber) -> Result<Level>
{
    use CellKind::Eraser;
    let levelInfo = match strand.0 {
        1 => LevelInfo{
            start:  StrandInfo{nodeCount: 6, edges: vec![(0,1), (1,2), (1,3), (0,4), (4,5)], mutables: vec![(2, Eraser), (4, Eraser)]},
            target: StrandInfo{nodeCount: 3, edges: vec![(0,1), (1,2)], mutables: vec![]},
            maxSplices: 0},
        2 => LevelInfo{
            start:  StrandInfo{nodeCount: 7, edges: vec![(0,1), (1,2), (1,3), (0,4), (4,5), (4,6)], mutables: vec![(2, Eraser)]},
            target: StrandInfo{nodeCount: 3, edges: vec![(0,1), (1,2)], mutables: vec![]},
            maxSplices: 1},
        3 => LevelInfo{
            start:  StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (1,3), (3,4), (3,5), (0,6), (6,7), (7,8), (7,9), (6,10)], mutables: vec![(5, Eraser)]},
            target: StrandInfo{nodeCount: 9,  edges: vec![(0,1), (1,2), (2,3), (2,4), (0,5), (5,6), (6,7), (6,8)], mutables: vec![]},
            maxSplices: 2}, // really 3, but it's angelic
        4 => LevelInfo{
            start:  StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (1,3), (0,4), (4,5), (5,6), (4,7), (7,8), (8,9), (7,10)], mutables: vec![(2, Eraser), (4, Eraser), (6, Eraser)]},
            target: StrandInfo{nodeCount: 6,  edges: vec![(0,1), (1,2), (2,3), (1,4), (4,5)], mutables: vec![]},
            maxSplices: 3},
        _ => bail!("Unsupported strand number: {}", strand.0)
    };
    Ok(makeLevelFrom(levelInfo))
}

fn makeLevelFrom(levelInfo: LevelInfo) -> Level
{
    let start = makeStrand(&levelInfo.start);
    let target = makeStrand(&levelInfo.target);
    Level{start, target, maxSplices: levelInfo.maxSplices}
}

fn makeStrand(strandInfo: &StrandInfo) -> Strand
{
    Strand::new(strandInfo.nodeCount, &strandInfo.edges, &strandInfo.mutables)
}

pub(crate) struct Level
{
    pub start: Strand,
    pub target: Strand,
    pub maxSplices: SpliceCount
}

pub(crate) type SpliceCount = u8;

#[derive(Clone, Copy)]
pub(crate) struct SequenceNumber(pub u8);

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct StrandNumber(pub u8);

struct LevelInfo
{
    start: StrandInfo,
    target: StrandInfo,
    maxSplices: SpliceCount
}

struct StrandInfo
{
    nodeCount: usize,
    edges: Vec<Edge>,
    mutables: Vec<(NodeId, CellKind)>
}
