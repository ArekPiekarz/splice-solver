use petgraph::{Direction, IntoWeightedEdge};
use petgraph::visit::{
    Data,
    Dfs,
    GetAdjacencyMatrix,
    GraphBase,
    GraphProp,
    IntoEdgeReferences,
    IntoNodeIdentifiers,
    IntoNodeReferences,
    NodeIndexable};
use petgraph::stable_graph::NodeIndex;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};


#[derive(Clone, Debug, Default)]
pub(crate) struct Strand(pub StrandInnerType);
type StrandInnerType = petgraph::stable_graph::StableDiGraph<StrandNode, StrandEdge>;

impl Strand
{
    pub fn add_node(&mut self, weight: StrandNode) -> NodeIndex
    {
        self.0.add_node(weight)
    }

    pub fn extend_with_edges<I>(&mut self, iterable: I)
        where
            I: IntoIterator,
            I::Item: IntoWeightedEdge<StrandEdge>,
            <I::Item as IntoWeightedEdge<StrandEdge>>::NodeId: Into<NodeIndex>
    {
        self.0.extend_with_edges(iterable)
    }
}

impl Eq for Strand {}

impl PartialEq for Strand {
    fn eq(&self, other: &Self) -> bool
    {
        // We assume the indices of the nodes do not matter, what matters is how they are connected,
        // meaning how many incoming and outgoing neighbors each node has.
        let mut selfDfs = Dfs::new(&self.0, NodeIndex::new(0));
        let mut otherDfs = Dfs::new(&other.0, NodeIndex::new(0));
        while let Some(selfNodeIndex) = selfDfs.next(&self.0) {
            let otherNodeIndex = match otherDfs.next(&other.0) {
                Some(index) => index,
                None => return false
            };
            if !isNeighborsCountTheSame(self, selfNodeIndex, other, otherNodeIndex, Direction::Incoming)
                || !isNeighborsCountTheSame(self, selfNodeIndex, other, otherNodeIndex, Direction::Outgoing) {
                return false;
            }
        }
        otherDfs.next(&other.0) == None
    }
}

fn isNeighborsCountTheSame(
    leftStrand: &Strand,
    leftNodeIndex: NodeIndex,
    rightStrand: &Strand,
    rightNodeIndex: NodeIndex,
    direction: Direction)
    -> bool
{
    let leftNeighborsCount = leftStrand.0.neighbors_directed(leftNodeIndex, direction).count();
    let rightNeighborsCount = rightStrand.0.neighbors_directed(rightNodeIndex, direction).count();
    leftNeighborsCount == rightNeighborsCount
}

impl Hash for Strand
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        self.0.adjacency_matrix().hash(state)
    }
}

impl<'a> IntoNodeReferences for &'a Strand
{
    type NodeRef = <&'a StrandInnerType as IntoNodeReferences>::NodeRef;
    type NodeReferences = <&'a StrandInnerType as IntoNodeReferences>::NodeReferences;
    fn node_references(self) -> Self::NodeReferences
    {
        self.0.node_references()
    }
}

impl<'a> IntoNodeIdentifiers for &'a Strand
{
    type NodeIdentifiers = <&'a StrandInnerType as IntoNodeIdentifiers>::NodeIdentifiers;
    fn node_identifiers(self) -> Self::NodeIdentifiers
    {
        self.0.node_identifiers()
    }
}

impl GraphBase for Strand
{
    type EdgeId = <StrandInnerType as GraphBase>::EdgeId;
    type NodeId = <StrandInnerType as GraphBase>::NodeId;
}

impl Data for &Strand
{
    type NodeWeight = <StrandInnerType as Data>::NodeWeight;
    type EdgeWeight = <StrandInnerType as Data>::EdgeWeight;
}

impl<'a> IntoEdgeReferences for &'a Strand
{
    type EdgeRef = <&'a StrandInnerType as IntoEdgeReferences>::EdgeRef;
    type EdgeReferences = <&'a StrandInnerType as IntoEdgeReferences>::EdgeReferences;

    fn edge_references(self) -> Self::EdgeReferences
    {
        self.0.edge_references()
    }
}

impl NodeIndexable for &Strand
{
    fn node_bound(self: &Self) -> usize
    {
        self.0.node_bound()
    }

    fn to_index(self: &Self, id: Self::NodeId) -> usize
    {
        self.0.to_index(id)
    }

    fn from_index(self: &Self, i: usize) -> Self::NodeId
    {
        self.0.from_index(i)
    }
}

impl<'a> GraphProp for &'a Strand
{
    type EdgeType = <&'a StrandInnerType as GraphProp>::EdgeType;

    fn is_directed(&self) -> bool
    {
        self.0.is_directed()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct StrandNode
{}

#[derive(Clone, Default)]
pub(crate) struct StrandEdge
{}

impl Debug for StrandEdge
{
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result
    {
        Ok(())
    }
}
