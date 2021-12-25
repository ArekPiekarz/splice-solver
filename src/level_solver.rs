use crate::level_maker::Level;
use crate::strand::{NodeId, Strand};

use pathfinding::directed::dijkstra::dijkstra;
use petgraph::visit::Dfs;

type SolutionNode = Strand;
type NodeAndCost = (SolutionNode, Cost);
type Cost = u32;


pub(crate) fn solveLevel(level: &Level) -> Option<Vec<Strand>>
{
    let result = dijkstra(
        &level.start, |node| makeSuccessors(node), |node| isGoalReached(node, &level.target));
    match result {
        Some((nodes, _cost)) => Some(nodes),
        None => None
    }
}

fn makeSuccessors(solutionNode: &SolutionNode) -> Vec<NodeAndCost>
{
    let mut successors = vec![];
    let mut dfs = Dfs::new(&solutionNode, SolutionNode::root());
    while let Some(strandNode) = dfs.next(&solutionNode) {
        successors.extend(makeNextStatesForStrandNode(strandNode, solutionNode));
    }
    successors.into_iter().map(|node| (node, 1)).collect()
}

fn makeNextStatesForStrandNode(nodeId: NodeId, strand: &Strand) -> Vec<Strand>
{
    let parent = match strand.parent(nodeId) {
        Some(parent) => parent,
        None => return vec![]
    };
    let newParents = findPotentialNewParents(nodeId, parent, strand);

    let mut result = vec![];
    for newParent in newParents {
        result.push(makeStrandWithNewParent(nodeId, newParent, strand));
    }
    if strand.childCount(parent) == 2 {
        result.push(makeStrandWithSwappedChildren(parent, strand));
    }
    result
}

fn findPotentialNewParents(nodeId: NodeId, parentId: NodeId, strand: &Strand) -> Vec<NodeId>
{
    let mut nodeIndices = strand.collectNodeIds();
    let mut excludedIndices = vec![nodeId];
    excludedIndices.extend(findChildrenRecursively(nodeId, strand));
    if strand.childCount(parentId) == 1 {
        excludedIndices.push(parentId);
    }
    nodeIndices = nodeIndices.into_iter().filter(|index| !excludedIndices.contains(index)).collect();
    nodeIndices = nodeIndices.into_iter().filter(|index| strand.childCount(*index) < 2).collect();
    nodeIndices
}

fn findChildrenRecursively(strandNodeIndex: NodeId, strand: &Strand) -> Vec<NodeId>
{
    let mut result = vec![];
    let mut dfs = Dfs::new(&strand, strandNodeIndex);
    while let Some(childIndex) = dfs.next(&strand) {
        if childIndex != strandNodeIndex {
            result.push(childIndex);
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

fn isGoalReached(node: &SolutionNode, target: &SolutionNode) -> bool
{
    node == target
}
