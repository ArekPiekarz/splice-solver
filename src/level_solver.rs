use crate::level_maker::{Level, SpliceCount};
use crate::strand::{CellKind, NodeId, Strand};

use pathfinding::directed::dijkstra::dijkstra;
use petgraph::visit::Dfs;


const NO_LAST_ACTION: Option<Action> = None;
const START_SPLICE_COUNT: SpliceCount = 0;

pub(crate) fn solveLevel(level: Level) -> Option<Vec<SolutionStep>>
{
    let startStep = SolutionStep::new(level.start, NO_LAST_ACTION, START_SPLICE_COUNT);
    let result = dijkstra(
        &startStep, |step| makeSuccessors(step, level.maxSplices), |step| isGoalReached(step, &level.target));
    result.map(|(nodes, _cost)| nodes)
}

fn makeSuccessors(solutionStep: &SolutionStep, maxSplices: SpliceCount) -> Vec<StepAndCost>
{
    let mut successors = vec![];
    let strand = &solutionStep.strand;
    let mut dfs = Dfs::new(strand, Strand::root());
    while let Some(strandNodeId) = dfs.next(strand) {
        successors.extend(makeSolutionStepsBySplicing(strandNodeId, solutionStep, maxSplices));
    }

    if let Some(newSolutionStep) = makeSolutionStepByMutation(solutionStep) {
        successors.push(newSolutionStep);
    }

    successors.into_iter().map(|node| (node, 1)).collect()
}

fn makeSolutionStepsBySplicing(nodeId: NodeId, solutionStep: &SolutionStep, maxSplices: SpliceCount) -> Vec<SolutionStep>
{
    let strand = &solutionStep.strand;
    let parent = match strand.parentId(nodeId) {
        Some(parent) => parent,
        None => return vec![]
    };

    let mut result = vec![];
    if solutionStep.spliceCount < maxSplices {
        let newSpliceCount = solutionStep.spliceCount + 1;
        let newParents = findPotentialNewParents(nodeId, parent, strand);
        for newParent in newParents {
            result.push(SolutionStep::new(
                makeStrandWithNewParent(nodeId, newParent, strand),
                Some(Action::ChangeParent { node: nodeId, oldParent: parent, newParent }),
                newSpliceCount));
        }
        if strand.childCount(parent) == 2 {
            result.push(SolutionStep::new(
                makeStrandWithSwappedChildren(parent, strand),
                Some(Action::SwapChildren { parent }),
                newSpliceCount));
        }
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
            [] => true,
            [childId] => strand.cellKind(*childId) != CellKind::Doubler && strand.cellKind(nodeId) != CellKind::Doubler,
            [_, _] => false,
            _ => panic!("Cell can't have more than two children")
        }).collect();
    outputNodeIds
}

fn findChildrenRecursively(nodeId: NodeId, strand: &Strand) -> Vec<NodeId>
{
    let mut result = vec![];
    let mut dfs = Dfs::new(strand, nodeId);
    while let Some(childId) = dfs.next(strand) {
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

fn makeSolutionStepByMutation(solutionStep: &SolutionStep) -> Option<SolutionStep>
{
    let mut newStrand = solutionStep.strand.clone();
    let mutatedNodeIds = newStrand.mutate();
    if !mutatedNodeIds.is_empty() {
        Some(SolutionStep::new(newStrand, Some(Action::Mutate{nodes: mutatedNodeIds}), solutionStep.spliceCount))
    } else {
        None
    }
}

fn isGoalReached(node: &SolutionStep, target: &Strand) -> bool
{
    node.strand.isEqualOnSurface(target)
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct SolutionStep
{
    pub strand: Strand,
    pub lastAction: Option<Action>,
    spliceCount: SpliceCount
}

impl SolutionStep
{
    fn new(strand: Strand, lastAction: Option<Action>, spliceCount: SpliceCount) -> Self
    {
        Self{strand, lastAction, spliceCount}
    }
}

type StepAndCost = (SolutionStep, Cost);
type Cost = u8;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum Action
{
    ChangeParent{node: NodeId, oldParent: NodeId, newParent: NodeId},
    SwapChildren{parent: NodeId},
    Mutate{nodes: Vec<NodeId>}
}
