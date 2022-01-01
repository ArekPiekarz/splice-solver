use crate::graph_utils::formatDotGraph;
use crate::level_solver::{SolutionStep, Splice};
use crate::strand::Strand;

use anyhow::{bail, Context, Result};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::{CellLayoutExt, GtkWindowExt, ToValue, TreeViewExt, WidgetExt};
use relm4::{send, AppUpdate, Model, RelmApp, Sender, Widgets};
use std::fs::write;
use std::process::Command;
use tempfile::{NamedTempFile, tempdir};


const EXPAND_IN_LAYOUT : bool = true;
const PRESERVE_ASPECT_RATIO: bool = true;

pub(crate) fn showSolution(solutionOpt: Option<Vec<SolutionStep>>) -> Result<()>
{
    match solutionOpt {
        Some(solution) => {
            match solution.len() {
                0 => bail!("Solution was found, but has no steps."),
                1 => bail!("Solution was found, but contains only 1 entry instead of at least 2 - start and end.\
                            As if the starting state was already solved."),
                _ => Ok(showValidSolution(solution)?)
            }
        },
        None => bail!("No solution was found.")
    }
}

fn showValidSolution(solution: Vec<SolutionStep>) -> Result<()>
{
    let solutionStepsVisuals = makeSolutionStepsVisuals(&solution)?;
    let model = AppModel{solutionSteps: solutionStepsVisuals, activeStep: 0};
    let relm = RelmApp::new(model);
    relm.run();
    Ok(())
}

fn makeSolutionStepsVisuals(solution: &[SolutionStep]) -> Result<Vec<SolutionStepVisual>>
{
    let mut output = vec![];
    for solutionStep in solution {
        let description = makeSolutionStepDescription(&solutionStep.lastSplice);
        let pixbuf = makeStrandPixbuf(&solutionStep.strand)?;
        output.push(SolutionStepVisual{description, pixbuf});
    }
    Ok(output)
}

fn makeSolutionStepDescription(spliceOpt: &Option<Splice>) -> String
{
    match spliceOpt {
        Some(splice) => {
            match splice {
                Splice::ChangeParent{node, oldParent, newParent} => {
                    format!("Change parent of node {} from {} to {}", node, oldParent, newParent)
                },
                Splice::SwapChildren{parent} => {
                    format!("Swap children of parent node {}", parent)
                }
            }
        },
        None => format!("Start")
    }
}

fn makeStrandPixbuf(strand: &Strand) -> Result<Pixbuf>
{
    let dotGraph = formatDotGraph(strand);
    let tempDir = tempdir()?;
    let dotGraphFile = NamedTempFile::new_in(&tempDir.path())?;
    let dotGraphFilePathStr = dotGraphFile.path().to_str().context("None")?;
    write(dotGraphFile.path(), dotGraph)?;

    let mut svgFilePath = tempDir.path().to_owned();
    svgFilePath.push("output.svg");
    let dotCommandOutput = Command::new("dot").args(["-Tsvg", dotGraphFilePathStr]).output()?;
    write(&svgFilePath, dotCommandOutput.stdout)?;
    Ok(Pixbuf::from_file_at_scale(&svgFilePath, 1920, 1080, PRESERVE_ASPECT_RATIO)?)
}

struct AppModel
{
    solutionSteps: Vec<SolutionStepVisual>,
    activeStep: usize
}

struct SolutionStepVisual
{
    description: String,
    pixbuf: Pixbuf
}

enum Event
{
    SelectionChanged(gtk::TreeSelection)
}

type AppComponents = ();

impl Model for AppModel
{
    type Msg = Event;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel
{
    fn update(&mut self, event: Event, _components: &AppComponents, _sender: Sender<Event>) -> bool
    {
        match event {
            Event::SelectionChanged(selection) => {
                let (rows, _model) = selection.selected_rows();
                self.activeStep = toRowIndex(&rows[0]);
            }
        };
        true
    }
}

#[must_use]
pub fn toRowIndex(rowPath: &gtk::TreePath) -> RowIndex
{
    rowPath.indices()[0].try_into().unwrap()
}

type RowIndex = usize;

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets
{
    fn pre_init()
    {
        let solutionStore = gtk::ListStore::new(&[glib::Type::STRING]);
        let column = 0;
        for step in &model.solutionSteps {
            solutionStore.set_value(&solutionStore.append(), column, &step.description.to_value());
        }
    }

    view! {
        gtk::ApplicationWindow {
            set_default_width: 900,
            set_default_height: 700,
            set_child: paned = Some(&gtk::Paned) {
                set_position: 240,
                set_start_child = &gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_child: listView = Some(&gtk::TreeView::with_model(&solutionStore)) {
                        append_column = &gtk::TreeViewColumn::new() {
                            set_title: "Solution steps"
                        }
                    }
                },
                set_end_child = &gtk::Image::from_pixbuf(Some(&model.solutionSteps[model.activeStep].pixbuf)) {}
            }
        }
    }

    fn post_init()
    {
        let renderer = gtk::CellRendererText::new();
        let index = 0;
        let column = listView.column(index).unwrap();
        column.pack_start(&renderer, EXPAND_IN_LAYOUT);
        column.add_attribute(&renderer, "text", index);
        column.set_resizable(true);

        listView.selection().connect_changed(move |selection|
            send!(sender, Event::SelectionChanged(selection.clone())));
    }

    fn manual_view(&mut self, model: &AppModel, _sender: Sender<AppMsg>)
    {
        self.paned.set_end_child(&gtk::Image::from_pixbuf(Some(&model.solutionSteps[model.activeStep].pixbuf)));
    }
}
