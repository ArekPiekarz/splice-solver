use crate::strand::Strand;

pub(crate) fn printGraph(graph: &Strand)
{
    println!("digraph {{");
    for edge in graph.collectEdges() {
        println!("    {} -> {}", edge.0, edge.1);
    }
    println!("}}");
}
