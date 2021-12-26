use fixedbitset::FixedBitSet;
use itertools::Itertools as _;
use petgraph::visit::{Dfs, GraphBase, IntoNeighbors, Visitable};


pub(crate) type NodeId = usize;
type Edge = (NodeId, NodeId);

#[derive(Clone, Debug, Hash)]
pub(crate) struct Strand
{
    nodes: Vec<Node>
}

impl Strand
{
    pub(crate) fn new(nodeCount: usize, edges: &[Edge]) -> Self
    {
        let mut newSelf = Self{nodes: vec![Node::default(); nodeCount]};
        for edge in edges {
            newSelf.addParent(edge.0, edge.1);
        }
        newSelf
    }

    pub(crate) fn root() -> NodeId
    {
        0
    }

    pub(crate) fn parent(&self, nodeId: NodeId) -> Option<NodeId>
    {
        self.nodes[nodeId].parentIdOpt
    }

    pub(crate) fn childCount(&self, nodeId: NodeId) -> usize
    {
        self.children(nodeId).len()
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
        let mut edges = vec![];
        let mut dfs = Dfs::new(self, Self::root());
        while let Some(nodeId) = dfs.next(self) {
            if let Some(parentId) = self.parent(nodeId) {
                edges.push((parentId, nodeId));
            }
        }
        edges
    }

    pub(crate) fn changeParent(&mut self, childId: NodeId, newParentId: NodeId)
    {
        assert_ne!(self.nodes[childId].parentIdOpt, Some(newParentId));
        self.removeParent(childId);
        self.addParent(newParentId, childId);
    }

    pub(crate) fn swapChildren(&mut self, nodeId: NodeId)
    {
        assert_eq!(self.childCount(nodeId), 2);
        self.nodes[nodeId].childrenIds.swap(0, 1);
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

    fn addParent(&mut self, parentId: NodeId, childId: NodeId)
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
}

impl Eq for Strand
{}

impl PartialEq for Strand {
    fn eq(&self, other: &Self) -> bool
    {
        // We assume the indices of the nodes do not matter, what matters is how they are connected,
        // meaning how many parents and children each node has.
        let mut selfDfs = Dfs::new(&self, Self::root());
        let mut otherDfs = Dfs::new(&other, Self::root());
        while let Some(selfNodeIndex) = selfDfs.next(&self) {
            let otherNodeIndex = match otherDfs.next(&other) {
                Some(index) => index,
                None => return false
            };
            if !isParentCountTheSame(self, selfNodeIndex, other, otherNodeIndex)
                || !isChildrenCountTheSame(self, selfNodeIndex, other, otherNodeIndex) {
                return false;
            }
        }
        otherDfs.next(&other) == None
    }
}

fn isParentCountTheSame(leftStrand: &Strand, leftNodeId: NodeId, rightStrand: &Strand, rightNodeId: NodeId) -> bool
{
    leftStrand.parent(leftNodeId).is_some() == rightStrand.parent(rightNodeId).is_some()
}

fn isChildrenCountTheSame(leftStrand: &Strand, leftNodeId: NodeId, rightStrand: &Strand, rightNodeId: NodeId) -> bool
{
    leftStrand.childCount(leftNodeId) == rightStrand.childCount(rightNodeId)
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
    parentIdOpt: Option<NodeId>,
    childrenIds: Vec<NodeId>
}
