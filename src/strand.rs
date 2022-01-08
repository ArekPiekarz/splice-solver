use fixedbitset::FixedBitSet;
use itertools::Itertools as _;
use petgraph::visit::{Dfs, GraphBase, IntoNeighbors, Visitable};
use std::collections::{BTreeMap, BTreeSet};


pub(crate) type NodeId = usize;
pub(crate) type Edge = (NodeId, NodeId);

#[derive(Clone, Debug, Hash)]
pub(crate) struct Strand
{
    nodes: Vec<Node>
}

impl Strand
{
    pub(crate) fn new(nodeCount: usize, edges: &[Edge], mutables: &[(NodeId,CellKind)]) -> Self
    {
        let mut newSelf = Self{nodes: vec![Node::default(); nodeCount]};
        for edge in edges {
            newSelf.connectParentToChild(edge.0, edge.1);
        }
        for mutable in mutables {
            let nodeId = mutable.0;
            let cellKind = mutable.1;
            newSelf.nodes[nodeId].cellKind = cellKind;
        }
        newSelf
    }

    pub(crate) fn root() -> NodeId
    {
        0
    }

    pub(crate) fn parentId(&self, nodeId: NodeId) -> Option<NodeId>
    {
        self.nodes[nodeId].parentIdOpt
    }

    pub(crate) fn childIds(&self, nodeId: NodeId) -> &[NodeId]
    {
        &self.nodes[nodeId].childrenIds
    }

    pub(crate) fn childCount(&self, nodeId: NodeId) -> usize
    {
        self.children(nodeId).len()
    }

    pub(crate) fn cellKind(&self, nodeId: NodeId) -> CellKind
    {
        self.nodes[nodeId].cellKind
    }

    pub(crate) fn collectNodeIds(&self) -> Vec<NodeId>
    {
        let mut output = vec![];
        for i in 0..self.nodes.len() {
            output.push(i);
        }
        output
    }

    pub(crate) fn collectEdges(&self) -> Vec<Edge>
    {
        self.collectEdgesFrom(Self::root())
    }

    pub(crate) fn changeParent(&mut self, childId: NodeId, newParentId: NodeId)
    {
        assert_ne!(self.nodes[childId].parentIdOpt, Some(newParentId));
        self.removeParent(childId);
        self.connectParentToChild(newParentId, childId);
    }

    pub(crate) fn swapChildren(&mut self, nodeId: NodeId)
    {
        assert_eq!(self.childCount(nodeId), 2);
        self.nodes[nodeId].childrenIds.swap(0, 1);
    }

    pub(crate) fn mutate(&mut self, nodeId: NodeId)
    {
        let node = &self.nodes[nodeId];
        match node.cellKind {
            CellKind::Normal => panic!("Cannot mutate a normal cell with id: {}", nodeId),
            CellKind::Doubler => self.mutateDoubler(nodeId)
        }
    }


    // private

    fn nodeCount(&self) -> usize
    {
        self.nodes.len()
    }

    fn children(&self, nodeId: NodeId) -> &[NodeId]
    {
        &self.nodes[nodeId].childrenIds
    }

    fn connectParentToChild(&mut self, parentId: NodeId, childId: NodeId)
    {
        assert_ne!(self.nodes[parentId].childrenIds.len(), 2);
        assert!(!self.nodes[parentId].childrenIds.contains(&childId));
        assert_eq!(self.nodes[childId].parentIdOpt, None);

        self.nodes[parentId].childrenIds.push(childId);
        self.nodes[childId].parentIdOpt = Some(parentId);
    }

    fn removeParent(&mut self, nodeId: NodeId)
    {
        let oldParentId = self.nodes[nodeId].parentIdOpt.unwrap();
        let oldChildIndex = self.nodes[oldParentId].childrenIds.iter().find_position(|id| **id == nodeId).unwrap().0;
        self.nodes[oldParentId].childrenIds.remove(oldChildIndex);
        self.nodes[nodeId].parentIdOpt = None;
    }

    fn collectEdgesFrom(&self, startNodeId: NodeId) -> Vec<Edge>
    {
        let mut edges = vec![];
        let mut dfs = Dfs::new(self, startNodeId);
        while let Some(nodeId) = dfs.next(self) {
            if nodeId == startNodeId {
                continue;
            }
            if let Some(parentId) = self.parentId(nodeId) {
                edges.push((parentId, nodeId));
            }
        }
        edges
    }

    fn mutateDoubler(&mut self, doublerNodeId: NodeId)
    {
        assert!(self.parentId(doublerNodeId).is_some());
        assert_eq!(self.childCount(self.parentId(doublerNodeId).unwrap()), 1);

        let edgesFromDoubler = self.collectEdgesFrom(doublerNodeId);
        let nodeCountFromDoubler = edgesFromDoubler.len() + 1;
        let originalNodeCount = self.nodes.len();
        for _ in 0..nodeCountFromDoubler {
            self.nodes.push(Node::default());
        }
        let mut oldNodeIdsSet = BTreeSet::new();
        for edge in &edgesFromDoubler {
            oldNodeIdsSet.insert(edge.0);
            oldNodeIdsSet.insert(edge.1);
        }
        let mut newNodeIdsMap = BTreeMap::new();
        for (index, oldNodeId) in oldNodeIdsSet.iter().enumerate() {
            newNodeIdsMap.insert(oldNodeId, originalNodeCount + index);
        }

        self.connectParentToChild(self.parentId(doublerNodeId).unwrap(), newNodeIdsMap[&doublerNodeId]);
        for edge in edgesFromDoubler {
            self.connectParentToChild(newNodeIdsMap[&edge.0], newNodeIdsMap[&edge.1]);
        }

        self.nodes[doublerNodeId].cellKind = CellKind::Normal;
    }
}

impl Eq for Strand
{}

impl PartialEq for Strand {
    fn eq(&self, other: &Self) -> bool
    {
        // We assume the indices of the nodes do not matter, what matters is how they are connected,
        // meaning how many parents and children each node has and what are the kinds of cells.
        let mut selfDfs = Dfs::new(&self, Self::root());
        let mut otherDfs = Dfs::new(&other, Self::root());
        while let Some(selfNodeIndex) = selfDfs.next(&self) {
            let otherNodeIndex = match otherDfs.next(&other) {
                Some(index) => index,
                None => return false
            };
            if !isParentCountTheSame(self, selfNodeIndex, other, otherNodeIndex)
                || !isChildrenCountTheSame(self, selfNodeIndex, other, otherNodeIndex)
                || !isCellKindTheSame(self, selfNodeIndex, other, otherNodeIndex) {
                return false;
            }
        }
        otherDfs.next(&other) == None
    }
}

fn isParentCountTheSame(leftStrand: &Strand, leftNodeId: NodeId, rightStrand: &Strand, rightNodeId: NodeId) -> bool
{
    leftStrand.parentId(leftNodeId).is_some() == rightStrand.parentId(rightNodeId).is_some()
}

fn isChildrenCountTheSame(leftStrand: &Strand, leftNodeId: NodeId, rightStrand: &Strand, rightNodeId: NodeId) -> bool
{
    leftStrand.childCount(leftNodeId) == rightStrand.childCount(rightNodeId)
}

fn isCellKindTheSame(leftStrand: &Strand, leftNodeId: NodeId, rightStrand: &Strand, rightNodeId: NodeId) -> bool
{
    leftStrand.cellKind(leftNodeId) == rightStrand.cellKind(rightNodeId)
}

impl GraphBase for Strand
{
    type EdgeId = ();
    type NodeId = NodeId;
}

impl Visitable for Strand
{
    type Map = FixedBitSet;
    fn visit_map(&self) -> FixedBitSet
    {
        FixedBitSet::with_capacity(self.nodeCount())
    }

    fn reset_map(&self, map: &mut Self::Map)
    {
        map.clear();
        map.grow(self.nodeCount());
    }
}

impl<'a> IntoNeighbors for &'a Strand
{
    type Neighbors = ChildNodesIterator;
    fn neighbors(self, nodeId: NodeId) -> Self::Neighbors
    {
        ChildNodesIterator::new(self.children(nodeId))
    }
}

pub(crate) struct ChildNodesIterator
{
    nodes: Vec<NodeId>,
    nextNodeId: usize
}

impl ChildNodesIterator
{
    fn new(nodes: &[NodeId]) -> Self
    {
        let mut nodes = nodes.to_vec();
        nodes.reverse();
        Self{nodes, nextNodeId: 0}
    }
}

impl Iterator for ChildNodesIterator
{
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.nextNodeId >= self.nodes.len() {
            return None;
        }

        let nextNodeId = self.nodes[self.nextNodeId];
        self.nextNodeId += 1;
        Some(nextNodeId)
    }
}

#[derive(Clone, Debug, Default, Hash)]
struct Node
{
    cellKind: CellKind,
    parentIdOpt: Option<NodeId>,
    childrenIds: Vec<NodeId>
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum CellKind
{
    Normal,
    Doubler
}

impl Default for CellKind
{
    fn default() -> Self
    {
        CellKind::Normal
    }
}
