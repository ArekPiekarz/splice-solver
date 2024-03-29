use arrayvec::ArrayVec;
use fixedbitset::FixedBitSet;
use itertools::Itertools as _;
use petgraph::visit::{Dfs, GraphBase, IntoNeighbors, Visitable};
use std::collections::BTreeMap;
use to_trait::To;

pub(crate) type NodeId = u8;
pub(crate) type Edge = (NodeId, NodeId);
type Depth = usize;


#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct Strand
{
    nodes: Vec<Option<Node>>
}

impl Strand
{
    pub(crate) fn new(nodeCount: usize, edges: &[Edge], mutables: &[(NodeId,CellKind)]) -> Self
    {
        let mut newSelf = Self{nodes: vec![Some(Node::default()); nodeCount]};
        for edge in edges {
            newSelf.connectParentToChild(edge.0, edge.1);
        }
        for mutable in mutables {
            let nodeId = mutable.0;
            let cellKind = mutable.1;
            newSelf.nodeAtMut(nodeId).cellKind = cellKind;
        }
        newSelf
    }

    pub(crate) fn root() -> NodeId
    {
        0
    }

    pub(crate) fn parentId(&self, nodeId: NodeId) -> Option<NodeId>
    {
        self.nodeAt(nodeId).parentIdOpt
    }

    pub(crate) fn childIds(&self, nodeId: NodeId) -> &[NodeId]
    {
        &self.nodeAt(nodeId).childrenIds
    }

    pub(crate) fn childCount(&self, nodeId: NodeId) -> usize
    {
        self.childIds(nodeId).len()
    }

    pub(crate) fn cellKind(&self, nodeId: NodeId) -> CellKind
    {
        self.nodeAt(nodeId).cellKind
    }

    pub(crate) fn collectNodeIds(&self) -> Vec<NodeId>
    {
        self.collectNodeIdsFrom(Self::root())
    }

    pub(crate) fn collectEdges(&self) -> Vec<Edge>
    {
        self.collectEdgesFrom(Self::root())
    }

    pub(crate) fn changeParent(&mut self, childId: NodeId, newParentId: NodeId)
    {
        debug_assert_ne!(self.parentId(childId), Some(newParentId));
        self.disconnectParentFromChild(childId);
        self.connectParentToChild(newParentId, childId);
    }

    pub(crate) fn swapChildren(&mut self, nodeId: NodeId)
    {
        debug_assert_eq!(self.childCount(nodeId), 2);
        self.nodeAtMut(nodeId).childrenIds.swap(0, 1);
    }

    pub(crate) fn mutate(&mut self) -> Vec<NodeId>
    {
        let mutableCellsIds = self.findMutableSpecialCellsIds();
        if mutableCellsIds.is_empty() {
            return vec![];
        }
        for cellId in &mutableCellsIds {
            match self.cellKind(*cellId) {
                CellKind::Doubler  => self.mutateDoubler(*cellId),
                CellKind::Extender => self.mutateExtender(*cellId),
                CellKind::Eraser   => self.mutateEraser(*cellId),
                CellKind::Normal   => panic!("Cannot mutate a normal cell with id: {}", cellId)
            }
        }
        mutableCellsIds
    }

    pub fn isEqualOnSurface(&self, other: &Self) -> bool
    {
        // We assume the indices of the nodes do not matter, what matters is how they are connected,
        // meaning how many children each node has and what are the kinds of cells.
        let mut selfDfs = Dfs::new(self, Self::root());
        let mut otherDfs = Dfs::new(other, Self::root());
        while let Some(selfNodeIndex) = selfDfs.next(self) {
            let otherNodeIndex = match otherDfs.next(other) {
                Some(index) => index,
                None => return false
            };
            if !isChildrenCountTheSame(self, selfNodeIndex, other, otherNodeIndex)
                || !isCellKindTheSame(self, selfNodeIndex, other, otherNodeIndex) {
                return false;
            }
        }
        otherDfs.next(other).is_none()
    }

    // private

    fn nodeAt(&self, nodeId: NodeId) -> &Node
    {
        self.nodes[nodeId.to::<usize>()].as_ref().unwrap()
    }

    fn nodeAtMut(&mut self, nodeId: NodeId) -> &mut Node
    {
        self.nodes[nodeId.to::<usize>()].as_mut().unwrap()
    }

    fn nodeCount(&self) -> usize
    {
        self.nodes.len()
    }

    fn connectParentToChild(&mut self, parentId: NodeId, childId: NodeId)
    {
        debug_assert_ne!(self.childCount(parentId), 2);
        debug_assert!(!self.childIds(parentId).contains(&childId));
        debug_assert_eq!(self.parentId(childId), None);

        self.nodeAtMut(parentId).childrenIds.push(childId);
        self.nodeAtMut(childId).parentIdOpt = Some(parentId);
    }

    fn disconnectParentFromChild(&mut self, childId: NodeId)
    {
        let oldParentId = self.parentId(childId).unwrap();
        let oldChildIndex = self.childIds(oldParentId).iter().find_position(|id| **id == childId).unwrap().0;
        self.nodeAtMut(oldParentId).childrenIds.remove(oldChildIndex);
        self.nodeAtMut(childId).parentIdOpt = None;
    }

    fn collectNodeIdsFrom(&self, startNodeId: NodeId) -> Vec<NodeId>
    {
        let mut output = Vec::with_capacity(self.nodeCount());
        let mut dfs = Dfs::new(self, startNodeId);
        while let Some(nodeId) = dfs.next(self) {
            output.push(nodeId);
        }
        output
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

    #[allow(clippy::comparison_chain)]
    fn findMutableSpecialCellsIds(&self) -> Vec<NodeId>
    {
        let mut shallowestDepth = usize::MAX;
        let mut output = vec![];
        for cellId in self.collectNodeIds() {
            if self.cellKind(cellId) == CellKind::Normal {
                continue;
            }

            let depth = self.calculateDepth(cellId);
            if depth < shallowestDepth {
                shallowestDepth = depth;
                output = vec![cellId];
            } else if depth == shallowestDepth {
                output.push(cellId);
            }
        }
        output
    }

    fn calculateDepth(&self, nodeId: NodeId) -> Depth
    {
        let mut depth = 0;
        let mut currentId = nodeId;
        while let Some(parentId) = self.parentId(currentId) {
            depth += 1;
            currentId = parentId
        }
        depth
    }

    fn mutateDoubler(&mut self, doublerNodeId: NodeId)
    {
        debug_assert!(self.parentId(doublerNodeId).is_some());
        debug_assert_eq!(self.childCount(self.parentId(doublerNodeId).unwrap()), 1);

        let edgesFromDoubler = self.collectEdgesFrom(doublerNodeId);
        let additionalNodeCountAfterMutation = edgesFromDoubler.len() + 1;
        let originalNodeCount = self.nodes.len();
        for _ in 0..additionalNodeCountAfterMutation {
            self.nodes.push(Some(Node::default()));
        }
        let mut oldNodeIdsList = vec![doublerNodeId];
        for edge in &edgesFromDoubler {
            oldNodeIdsList.push(edge.1);
        }
        let mut newNodeIdsMap = BTreeMap::new();
        for (index, oldNodeId) in oldNodeIdsList.iter().enumerate() {
            newNodeIdsMap.insert(oldNodeId, (originalNodeCount + index).try_to::<NodeId>().unwrap());
        }

        self.connectParentToChild(self.parentId(doublerNodeId).unwrap(), newNodeIdsMap[&doublerNodeId]);
        for edge in edgesFromDoubler {
            self.connectParentToChild(newNodeIdsMap[&edge.0], newNodeIdsMap[&edge.1]);
        }

        self.nodeAtMut(doublerNodeId).cellKind = CellKind::Normal;
        let oldMutableCellIds =
            oldNodeIdsList.iter().dropping(1).filter(|id| self.cellKind(**id) != CellKind::Normal).collect_vec();
        for oldMutableCellId in oldMutableCellIds {
            self.nodeAtMut(newNodeIdsMap[oldMutableCellId]).cellKind = self.cellKind(*oldMutableCellId);
        }
    }

    fn mutateExtender(&mut self, extenderNodeId: NodeId)
    {
        self.nodes.push(Some(Node::default()));
        let newNodeId = (self.nodeCount() - 1).try_to::<NodeId>().unwrap();
        self.nodeAtMut(extenderNodeId).cellKind = CellKind::Normal;
        let childIds = self.childIds(extenderNodeId).to_vec();
        match childIds[..] {
            [] => self.connectParentToChild(extenderNodeId, newNodeId),
            [childId] => {
                self.changeParent(childId, newNodeId);
                self.connectParentToChild(extenderNodeId, newNodeId);
            },
            [childId1, childId2] => {
                self.changeParent(childId1, newNodeId);
                self.changeParent(childId2, newNodeId);
                self.connectParentToChild(extenderNodeId, newNodeId);
            },
            _ => panic!("Cell cannot have more than 2 children")
        }
    }

    fn mutateEraser(&mut self, eraserNodeId: NodeId)
    {
        let nodeIdsToErase = self.collectNodeIdsFrom(eraserNodeId);
        self.disconnectParentFromChild(eraserNodeId);
        for nodeId in nodeIdsToErase {
            self.nodes[nodeId.to::<usize>()] = None;
        }
    }
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
        ChildNodesIterator::new(self.childIds(nodeId))
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
struct Node
{
    cellKind: CellKind,
    parentIdOpt: Option<NodeId>,
    childrenIds: ArrayVec<NodeId, 2>
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum CellKind
{
    Normal,
    Doubler,
    Extender,
    Eraser
}

impl Default for CellKind
{
    fn default() -> Self
    {
        CellKind::Normal
    }
}
