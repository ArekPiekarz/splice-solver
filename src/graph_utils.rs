use crate::graph_types::Strand;

pub(crate) fn printGraph(graph: &Strand)
{
    println!("{:?}", petgraph::dot::Dot::with_config(&graph, &[petgraph::dot::Config::NodeIndexLabel]));
}
