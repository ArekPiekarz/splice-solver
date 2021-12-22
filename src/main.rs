#![allow(non_snake_case)]

mod graph_types;
mod graph_utils;
mod levels;
mod level_solver;

use crate::graph_utils::printGraph;
use crate::levels::{makeSequence1Strand1, makeSequence1Strand2};
use crate::level_solver::solveLevel;

fn main() {
    let level = makeSequence1Strand1();
    let solution = solveLevel(&level);
    println!("solution 1 found:");
    for step in &solution {
        printGraph(&step);
    }

    let level = makeSequence1Strand2();
    let solution = solveLevel(&level);
    println!("solution 2 found:");
    for step in &solution {
        printGraph(&step);
    }
}
