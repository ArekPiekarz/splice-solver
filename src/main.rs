#![allow(non_snake_case)]

mod graph_utils;
mod gui;
mod level_maker;
mod level_solver;
mod strand;

use crate::gui::makeGui;

use mimalloc::MiMalloc;

#[global_allocator]
static ALLOCATOR: MiMalloc = MiMalloc;


fn main()
{
    makeGui()
}
