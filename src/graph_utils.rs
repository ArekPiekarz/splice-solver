use crate::strand::{CellKind, NodeId, Strand};


pub(crate) fn formatDotGraph(strand: &Strand) -> String
{
    let mut output = String::new();
    output.push_str("digraph {\n");
    for edge in strand.collectEdges() {
        output.push_str(&format!("    {} -> {}\n", formatNode(edge.0, strand), formatNode(edge.1, strand)));
    }
    output.push('}');
    output
}

fn formatNode(nodeId: NodeId, strand: &Strand) -> String
{
    match strand.cellKind(nodeId) {
        CellKind::Normal => format!("{}", nodeId),
        CellKind::Doubler => format!("\"{} (doubler)\"", nodeId),
        CellKind::Extender => format!("\"{} (extender)\"", nodeId),
        CellKind::Eraser => format!("\"{} (eraser)\"", nodeId)
    }
}
