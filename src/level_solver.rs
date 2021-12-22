use crate::graph_types::{Strand, StrandEdge};
use crate::levels::Level;

use itertools::Itertools as _;
use pathfinding::directed::dijkstra::dijkstra;
use petgraph::Direction;
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::Dfs;

type SolutionNode = Strand;
type NodeAndCost = (SolutionNode, Cost);
type Cost = u32;


pub(crate) fn solveLevel(level: &Level) -> Vec<Strand>
{
    let result = dijkstra(
        &level.start, |node| makeSuccessors(node), |node| isGoalReached(node, &level.target));
    match result {
        Some((nodes, _cost)) => nodes,
        None => vec![]
    }
}

fn makeSuccessors(solutionNode: &SolutionNode) -> Vec<NodeAndCost>
{
    let mut successors = vec![];
    let mut dfs = Dfs::new(&solutionNode.0, solutionNode.0.node_indices().next().unwrap());
    while let Some(strandNode) = dfs.next(&solutionNode.0) {
        successors.extend(makeNextStatesForStrandNode(strandNode, solutionNode));
    }
    successors.into_iter().map(|node| (node, 1)).collect()
}

fn makeNextStatesForStrandNode(strandNodeIndex: NodeIndex, strand: &Strand) -> Vec<Strand>
{
    let mut parents = strand.0.neighbors_directed(strandNodeIndex, Direction::Incoming);
    let parent = match parents.next() {
        Some(parent) => parent,
        None => return vec![]
    };
    let newParents = findPotentialNewParents(strandNodeIndex, parent, strand);

    let mut result = vec![];
    for newParent in newParents {
        result.push(makeStrandWithNewParent(strandNodeIndex, parent, newParent, strand));
    }
    result
}

fn findPotentialNewParents(strandNodeIndex: NodeIndex, parentIndex: NodeIndex, strand: &Strand) -> Vec<NodeIndex>
{
    let mut nodeIndices = strand.0.node_indices().collect_vec();
    let mut excludedIndices = vec![strandNodeIndex, parentIndex];
    excludedIndices.extend(findChildrenRecursively(strandNodeIndex, strand));
    nodeIndices = nodeIndices.into_iter().filter(|index| !excludedIndices.contains(index)).collect();
    nodeIndices.into_iter().filter(|index| countChildren(*index, strand) < 2).collect()
}

fn findChildrenRecursively(strandNodeIndex: NodeIndex, strand: &Strand) -> Vec<NodeIndex>
{
    let mut result = vec![];
    let mut dfs = Dfs::new(&strand.0, strandNodeIndex);
    while let Some(childIndex) = dfs.next(&strand.0) {
        result.push(childIndex);
    }
    result
}

fn countChildren(index: NodeIndex, strand: &Strand) -> usize
{
    strand.0.neighbors_directed(index, Direction::Outgoing).count()
}

fn makeStrandWithNewParent(node: NodeIndex, oldParent: NodeIndex, newParent: NodeIndex, strand: &Strand) -> Strand
{
    let mut newStrand = strand.clone();
    let edge = newStrand.0.find_edge(oldParent, node).unwrap();
    newStrand.0.remove_edge(edge);
    newStrand.0.add_edge(newParent, node, StrandEdge{});
    newStrand
}

fn isGoalReached(node: &SolutionNode, target: &SolutionNode) -> bool
{
    node == target
}

#[allow(dead_code)]
fn formatNeighbors(nodeIndex: NodeIndex, strand: &Strand) -> String
{
    let mut result = String::new();
    for neighbor in strand.0.neighbors(nodeIndex) {
        result.push_str(&format!("{:?}, ", neighbor));
    }
    result
}
