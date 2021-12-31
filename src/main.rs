#![allow(non_snake_case)]

mod graph_utils;
mod gui;
mod level_maker;
mod level_solver;
mod strand;

use crate::gui::showSolution;
use crate::level_maker::{makeLevel, SequenceNumber, StrandNumber};
use crate::level_solver::solveLevel;

use anyhow::Result;
use clap::Parser;


fn main() -> Result<()>
{
    let args = Args::parse();
    let level = makeLevel(SequenceNumber(args.sequence), StrandNumber(args.strand))?;
    let solution = solveLevel(level);
    showSolution(solution)?;
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
