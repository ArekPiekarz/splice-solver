#![allow(non_snake_case)]

mod graph_utils;
mod gui;
mod level_maker;
mod level_solver;
mod strand;

use crate::gui::makeGui;

use anyhow::Result;


fn main() -> Result<()>
{
    makeGui()
}
