#![allow(non_snake_case)]

mod graph_types;
mod graph_utils;
mod level_maker;
mod level_solver;

use crate::graph_types::Strand;
use crate::graph_utils::printGraph;
use crate::level_maker::{makeLevel, SequenceNumber, StrandNumber};
use crate::level_solver::solveLevel;

use anyhow::{bail, Result};
use clap::Parser;


fn main() -> Result<()>
{
    let args = Args::parse();
    let level = makeLevel(SequenceNumber(args.sequence), StrandNumber(args.strand))?;
    let solution = solveLevel(&level);
    printSolution(&solution)?;
    Ok(())
}

#[derive(Parser)]
struct Args {
    /// Sequence number
    #[clap(short, long)]
    sequence: u8,

    /// Strand number
    #[clap(short = 't', long)]
    strand: u8,
}

fn printSolution(solution: &Option<Vec<Strand>>) -> Result<()>
{
    match solution {
        Some(solution) => {
            match solution.len() {
                0 => bail!("Solution was found, but has no steps."),
                1 => bail!("Solution was found, but contains only 1 entry instead of at least 2 - start and end."),
                _ => Ok(printValidSolution(solution))
            }
        },
        None => bail!("No solution was found.")
    }
}

fn printValidSolution(solution: &[Strand])
{
    let splicesCount = solution.len() - 1; // do not count the start state
    println!("Solution was found in {} splice{}:",
             splicesCount,
             if splicesCount == 1 { "" } else { "s" });
    for step in solution {
        printGraph(&step);
    }
}
