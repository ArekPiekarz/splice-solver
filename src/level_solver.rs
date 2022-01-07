use crate::level_maker::{Level, SpliceCount};
use crate::strand::{CellKind, NodeId, Strand};

use pathfinding::directed::dijkstra::dijkstra;
use petgraph::visit::Dfs;


const NO_LAST_SPLICE: Option<Splice> = None;
const START_SPLICE_COUNT: SpliceCount = 0;

pub(crate) fn solveLevel(level: Level) -> Option<Vec<SolutionStep>>
{
    let startStep = SolutionStep::new(level.start, NO_LAST_SPLICE, START_SPLICE_COUNT);
    let result = dijkstra(
        &startStep, |step| makeSuccessors(step, level.maxSplices), |step| isGoalReached(step, &level.target));
    match result {
        Some((nodes, _cost)) => Some(nodes),
        None => None
    }
}

fn makeSuccessors(solutionStep: &SolutionStep, maxSplices: SpliceCount) -> Vec<StepAndCost>
{
    if solutionStep.spliceCount >= maxSplices {
        return vec![];
    }

    let mut successors = vec![];
    let strand = &solutionStep.strand;
    let mut dfs = Dfs::new(strand, Strand::root());
    while let Some(strandNode) = dfs.next(strand) {
        successors.extend(makeNextStatesForStrandNode(strandNode, solutionStep));
    }
    successors.into_iter().map(|node| (node, 1)).collect()
}

fn makeNextStatesForStrandNode(nodeId: NodeId, solutionStep: &SolutionStep) -> Vec<SolutionStep>
{
    let strand = &solutionStep.strand;
    let parent = match strand.parentId(nodeId) {
        Some(parent) => parent,
        None => return vec![]
    };
    let newParents = findPotentialNewParents(nodeId, parent, strand);

    let mut result = vec![];
    let newSpliceCount = solutionStep.spliceCount + 1;
    for newParent in newParents {
        result.push(SolutionStep::new(
            makeStrandWithNewParent(nodeId, newParent, strand),
            Some(Splice::ChangeParent{node: nodeId, oldParent: parent, newParent}),
            newSpliceCount));
    }
    if strand.childCount(parent) == 2 {
        result.push(SolutionStep::new(
            makeStrandWithSwappedChildren(parent, strand),
            Some(Splice::SwapChildren{parent}),
            newSpliceCount));
    }

    if strand.cellKind(nodeId) == CellKind::Doubler {
        result.push(SolutionStep::new(
            makeStrandWithMutation(nodeId, strand),
            Some(Splice::Mutate{nodes: vec![nodeId]}),
            newSpliceCount));
    }

    result
}

fn findPotentialNewParents(nodeId: NodeId, parentId: NodeId, strand: &Strand) -> Vec<NodeId>
{
    let mut outputNodeIds = strand.collectNodeIds();
    let mut excludedIndices = vec![nodeId, parentId];
    excludedIndices.extend(findChildrenRecursively(nodeId, strand));

    outputNodeIds = outputNodeIds.into_iter()
        .filter(|id| !excludedIndices.contains(id))
        .filter(|id| match strand.childIds(*id) {
            [_, _] => false,
            [childId] => strand.cellKind(*childId) == CellKind::Normal,
            _ => true
        }).collect();
    outputNodeIds
}

fn findChildrenRecursively(nodeId: NodeId, strand: &Strand) -> Vec<NodeId>
{
    let mut result = vec![];
    let mut dfs = Dfs::new(&strand, nodeId);
    while let Some(childId) = dfs.next(&strand) {
        if childId != nodeId {
            result.push(childId);
        }
    }
    result
}

fn makeStrandWithNewParent(nodeId: NodeId, newParentId: NodeId, strand: &Strand) -> Strand
{
    let mut newStrand = strand.clone();
    newStrand.changeParent(nodeId, newParentId);
    newStrand
}

fn makeStrandWithSwappedChildren(parentId: NodeId, strand: &Strand) -> Strand
{
    let mut newStrand = strand.clone();
    newStrand.swapChildren(parentId);
    newStrand
}

fn makeStrandWithMutation(nodeId: NodeId, strand: &Strand) -> Strand
{
    let mut newStrand = strand.clone();
    newStrand.mutate(nodeId);
    newStrand
}

fn isGoalReached(node: &SolutionStep, target: &Strand) -> bool
{
    node.strand == *target
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct SolutionStep
{
    pub strand: Strand,
    pub lastSplice: Option<Splice>,
    spliceCount: SpliceCount
}

impl SolutionStep
{
    fn new(strand: Strand, lastSplice: Option<Splice>, spliceCount: SpliceCount) -> Self
    {
        Self{strand, lastSplice, spliceCount}
    }
}

type StepAndCost = (SolutionStep, Cost);
type Cost = u8;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum Splice
{
    ChangeParent{node: NodeId, oldParent: NodeId, newParent: NodeId},
    SwapChildren{parent: NodeId},
    Mutate{nodes: Vec<NodeId>}
}
