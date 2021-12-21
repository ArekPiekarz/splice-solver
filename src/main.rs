#![allow(non_snake_case)]

mod graph_types;
mod graph_utils;
mod levels;
mod level_solver;

use crate::graph_utils::printGraph;
use crate::levels::makeSequence1Strand1;
use crate::level_solver::solveLevel;

fn main() {
    let level = makeSequence1Strand1();
    let solution = solveLevel(&level);
    println!("solution 1 found:");
    for step in &solution {
        printGraph(&step);
    }
}
