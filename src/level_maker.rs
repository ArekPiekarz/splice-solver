use crate::strand::{NodeId, Strand};

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
        _ => bail!("Unsupported sequence number: {}", sequence.0)
    }
}

fn makeStrandInSequence1(strand: StrandNumber) -> Result<Level>
{
    let levelInfo = match strand.0 {
        1 => LevelInfo{
            start:  StrandInfo{nodeCount: 3, edges: vec![(0,1), (1,2)]},
            target: StrandInfo{nodeCount: 3, edges: vec![(0,1), (0,2)]},
            maxMoves: 1},
        2 => LevelInfo{
            start:  StrandInfo{nodeCount: 5, edges: vec![(0,1), (1,2), (2,3), (1,4)]},
            target: StrandInfo{nodeCount: 5, edges: vec![(0,1), (1,2), (0,3), (3,4)]},
            maxMoves: 1},
        3 => LevelInfo{
            start:  StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (1,3), (3,4), (3,5), (0,6), (6,7), (6,8), (8,9), (8,10)]},
            target: StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (2,3), (3,4), (3,5), (0,6), (6,7), (7,8), (8,9), (8,10)]},
            maxMoves: 2},
        4 => LevelInfo{
            start:  StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (2,3), (2,4), (4,5), (1,6), (6,7), (7,8), (6,9), (0,10)]},
            target: StrandInfo{nodeCount: 11, edges: vec![(0,1), (1,2), (2,3), (3,4), (2,5), (0,6), (6,7), (7,8), (7,9), (9,10)]},
            maxMoves: 1},
        5 => LevelInfo{
            start:  StrandInfo{nodeCount: 10, edges: vec![(0,1), (1,2), (1,3), (0,4), (4,5), (4,6), (6,7), (7,8), (7,9)]},
            target: StrandInfo{nodeCount: 10, edges: vec![(0,1), (1,2), (2,3), (3,4), (3,5), (1,6), (6,7), (7,8), (7,9)]},
            maxMoves: 1},
        6 => LevelInfo{
            start:  StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (1,3), (3,4), (4,5), (4,6), (0,7), (7,8), (8,9), (9,10), (9,11),  (7,12)]},
            target: StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (2,3), (3,4), (3,5), (1,6), (0,7), (7,8), (7,9), (9,10), (10,11), (10,12)]},
            maxMoves: 1},
        7 => LevelInfo{
            start:  StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (2,3), (3,4), (3,5), (5,6), (5,7), (2,8), (8,9), (9,10), (9,11), (8,12)]},
            target: StrandInfo{nodeCount: 13, edges: vec![(0,1), (1,2), (2,3), (2,4), (4,5), (4,6), (0,7), (7,8), (8,9), (9,10), (9,11), (8,12)]},
            maxMoves: 2},
        _ => bail!("Unsupported strand number: {}", strand.0)
    };
    Ok(makeLevelFrom(levelInfo))
}

fn makeLevelFrom(levelInfo: LevelInfo) -> Level
{
    let start = makeStrand(&levelInfo.start);
    let target = makeStrand(&levelInfo.target);
    Level{start, target, maxMoves: levelInfo.maxMoves}
}

fn makeStrand(strandInfo: &StrandInfo) -> Strand
{
    Strand::new(strandInfo.nodeCount, &strandInfo.edges)
}

pub(crate) struct Level
{
    pub start: Strand,
    pub target: Strand,
    maxMoves: NumberOfMoves
}

type NumberOfMoves = u32;

pub(crate) struct SequenceNumber(pub u8);
pub(crate) struct StrandNumber(pub u8);

struct LevelInfo
{
    start: StrandInfo,
    target: StrandInfo,
    maxMoves: u32
}

struct StrandInfo
{
    nodeCount: usize,
    edges: Vec<(NodeId,NodeId)>
}