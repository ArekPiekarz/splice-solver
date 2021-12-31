use crate::strand::Strand;


pub(crate) fn formatDotGraph(graph: &Strand) -> String
{
    let mut output = String::new();
    output.push_str("digraph {\n");
    for edge in graph.collectEdges() {
        output.push_str(&format!("    {} -> {}\n", edge.0, edge.1));
    }
    output.push_str("}");
    output
}
