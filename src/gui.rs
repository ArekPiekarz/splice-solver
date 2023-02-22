#![allow(clippy::enum_variant_names)]

use crate::graph_utils::formatDotGraph;
use crate::level_maker::{makeLevel, SequenceNumber, StrandNumber};
use crate::level_solver::{Action, solveLevel, SolutionStep};
use crate::strand::{NodeId, Strand};

use anyhow::{bail, Context, Result};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::{BoxExt, GridExt, GtkWindowExt, ToValue, TreeModelExt, TreeViewExt, WidgetExt};
use relm4::gtk;
use std::fs::write;
use std::process::Command;
use tempfile::{NamedTempFile, tempdir};
use to_trait::To;


const EXPAND_IN_LAYOUT : bool = true;
const PRESERVE_ASPECT_RATIO: bool = true;
const SPACING_I32: i32 = 5;
const SPACING_U32: u32 = 5;
const ZEROTH_COLUMN_I32: i32 = 0;
const ZEROTH_COLUMN_U32: u32 = 0;

pub(crate) fn makeGui()
{
    let app = relm4::RelmApp::new("global.splice-solver");
    app.run::<AppModel>(());
}

fn makeSolutionVisuals(solutionOpt: Option<Vec<SolutionStep>>) -> Result<Vec<SolutionStepVisual>>
{
    match solutionOpt {
        Some(solution) => {
            match solution.len() {
                0 => bail!("Solution was found, but has no steps."),
                1 => bail!("Solution was found, but contains only 1 entry instead of at least 2 - start and end.\
                            As if the starting state was already solved."),
                _ => makeValidSolutionVisuals(&solution)
            }
        },
        None => bail!("No solution was found.")
    }
}

fn makeValidSolutionVisuals(solution: &[SolutionStep]) -> Result<Vec<SolutionStepVisual>>
{
    let mut output = vec![];
    for solutionStep in solution {
        let description = makeSolutionStepDescription(&solutionStep.lastAction);
        let pixbuf = makeStrandPixbuf(&solutionStep.strand)?;
        output.push(SolutionStepVisual{description, pixbuf});
    }
    Ok(output)
}

fn makeSolutionStepDescription(actionOpt: &Option<Action>) -> String
{
    match actionOpt {
        Some(action) => {
            match action {
                Action::ChangeParent{node, oldParent, newParent} => {
                    format!("Change parent of node {} from {} to {}", node, oldParent, newParent)
                },
                Action::SwapChildren{parent} => {
                    format!("Swap children of parent node {}", parent)
                },
                Action::Mutate{nodes} => {
                    makeMutateStepDescription(nodes)
                }
            }
        },
        None => "Start".into()
    }
}

fn makeMutateStepDescription(nodes: &[NodeId]) -> String
{
    match nodes {
        [] => panic!("Nodes to mutate cannot be empty"),
        [nodeId] => format!("Mutate node {}", nodeId),
        [_, ..] => format!("Mutate nodes {}", formatNodesIntoList(nodes))
    }
}

fn formatNodesIntoList(nodes: &[NodeId]) -> String
{
    let mut output = String::new();
    for (index, nodeId) in nodes.iter().enumerate() {
        output.push_str(&format!("{}", nodeId));
        match nodes.len() - 1 - index {
            0 => (),
            1 => output.push_str(" and "),
            _ => output.push_str(", ")
        }
    }
    output
}

fn makeStrandPixbuf(strand: &Strand) -> Result<Pixbuf>
{
    let dotGraph = formatDotGraph(strand);
    let tempDir = tempdir()?;
    let dotGraphFile = NamedTempFile::new_in(tempDir.path())?;
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
    sequenceNumber: SequenceNumber,
    strandNumber: StrandNumber,
    maxStrandNumber: StrandNumber,
    solutionSteps: Vec<SolutionStepVisual>,
    activeStep: usize,
    solutionStore: gtk::ListStore,
}

#[derive(Debug)]
struct SolutionStepVisual
{
    description: String,
    pixbuf: Pixbuf,
}

#[derive(Debug)]
enum Event
{
    SelectionChanged(gtk::TreeSelection),
    SequenceNumberChanged(i32),
    StrandNumberChanged(i32),
}

impl AppModel
{
    fn new() -> Self
    {
        let mut newSelf = Self{
            sequenceNumber: SequenceNumber(1),
            strandNumber: StrandNumber(1),
            maxStrandNumber: StrandNumber(7),
            solutionSteps: vec![],
            activeStep: 0,
            solutionStore: gtk::ListStore::new(&[glib::Type::STRING])};
        newSelf.onLevelChanged();
        newSelf
    }

    fn onSelectionChanged(&mut self, selection: &gtk::TreeSelection)
    {
        let (rows, _model) = selection.selected_rows();
        if rows.is_empty() {
            return;
        }
        self.activeStep = toRowIndex(&rows[0]);
    }

    fn onSequenceNumberChanged(&mut self, value: i32)
    {
        self.sequenceNumber = SequenceNumber(value.try_into().unwrap());
        self.strandNumber = StrandNumber(1);
        self.maxStrandNumber = StrandNumber(match self.sequenceNumber.0 {
            1..=4 => 7,
            5 => 2,
            number => panic!("Unsupported sequence number: {}", number)
        });
        self.onLevelChanged();
    }

    fn onStrandNumberChanged(&mut self, value: i32)
    {
        let newStrandNumber = StrandNumber(value.try_into().unwrap());
        if self.strandNumber == newStrandNumber {
            return;
        }
        self.strandNumber = newStrandNumber;
        self.onLevelChanged();
    }

    fn onLevelChanged(&mut self)
    {
        let level = makeLevel(self.sequenceNumber, self.strandNumber).unwrap();
        let solution = solveLevel(level);
        let solutionVisuals = makeSolutionVisuals(solution).unwrap();
        self.solutionSteps = solutionVisuals;
        self.activeStep = 0;
        self.solutionStore.clear();
        for step in &self.solutionSteps {
            self.solutionStore.set_value(&self.solutionStore.append(), ZEROTH_COLUMN_U32, &step.description.to_value());
        }
    }
}

struct AppWidgets
{
    paned: gtk::Paned,
    strandSpinButton: gtk::SpinButton,
    listView: gtk::TreeView,
}

impl relm4::SimpleComponent for AppModel
{
    type Input = Event;
    type Output = ();
    type Init = ();
    type Root = gtk::ApplicationWindow;
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root
    {
        let appWindow = gtk::ApplicationWindow::default();
        appWindow.set_default_width(900);
        appWindow.set_default_height(700);
        appWindow
    }

    fn init(_: Self::Init, appWindow: &Self::Root, sender: relm4::ComponentSender<Self>) -> relm4::ComponentParts<Self>
    {
        let model = AppModel::new();

        let sequenceSpinButton = gtk::SpinButton::with_range(1.0, 5.0, 1.0);
        sequenceSpinButton.set_can_focus(false);
        let sender2 = sender.clone();
        sequenceSpinButton.connect_value_changed(move |spinButton| {
            sender2.input(Event::SequenceNumberChanged(spinButton.value_as_int()));
        });

        let strandSpinButton = gtk::SpinButton::with_range(1.0, 7.0, 1.0);
        strandSpinButton.set_can_focus(false);
        let sender3 = sender.clone();
        strandSpinButton.connect_value_changed(move |spinButton| {
            sender3.input(Event::StrandNumberChanged(spinButton.value_as_int()));
        });

        let parametersGrid = gtk::Grid::default();
        parametersGrid.set_row_spacing(SPACING_U32);
        parametersGrid.set_column_spacing(SPACING_U32);
        parametersGrid.attach(&gtk::Label::new(Some("Sequence")), 0, 0, 1, 1);
        parametersGrid.attach(&sequenceSpinButton, 1, 0, 1, 1);
        parametersGrid.attach(&gtk::Label::new(Some("Strand")), 0, 1, 1, 1);
        parametersGrid.attach(&strandSpinButton, 1, 1, 1, 1);

        let listViewColumn = gtk::TreeViewColumn::default();
        listViewColumn.set_title("Solution steps");
        let listView = gtk::TreeView::with_model(&model.solutionStore);
        listView.append_column(&listViewColumn);
        listView.selection().connect_changed(move |selection|
            sender.input(Event::SelectionChanged(selection.clone())));

        let renderer = gtk::CellRendererText::new();
        let column = listView.column(ZEROTH_COLUMN_I32).unwrap();
        column.pack_start(&renderer, EXPAND_IN_LAYOUT);
        column.add_attribute(&renderer, "text", ZEROTH_COLUMN_I32);
        column.set_resizable(true);

        let scrolledWindow = gtk::ScrolledWindow::new();
        scrolledWindow.set_vexpand(true);
        scrolledWindow.set_child(Some(&listView));

        let leftPaneBox = gtk::Box::new(gtk::Orientation::Vertical, SPACING_I32);
        leftPaneBox.append(&parametersGrid);
        leftPaneBox.append(&scrolledWindow);

        let paned = gtk::Paned::default();
        paned.set_position(240);
        paned.set_start_child(Some(&leftPaneBox));

        appWindow.set_child(Some(&paned));

        let widgets = AppWidgets{paned, strandSpinButton, listView};
        relm4::ComponentParts{model, widgets}
    }

    fn update(&mut self, event: Self::Input, _sender: relm4::ComponentSender<Self>)
    {
        match event {
            Event::SelectionChanged(selection) => self.onSelectionChanged(&selection),
            Event::SequenceNumberChanged(value) => self.onSequenceNumberChanged(value),
            Event::StrandNumberChanged(value) => self.onStrandNumberChanged(value)
        };
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: relm4::ComponentSender<Self>)
    {
        if widgets.strandSpinButton.value_as_int() != self.strandNumber.0.to::<i32>() {
            widgets.strandSpinButton.set_value(self.strandNumber.0.into());
        }

        if widgets.strandSpinButton.range().1 != self.maxStrandNumber.0.into() {
            widgets.strandSpinButton.set_range(1.0, self.maxStrandNumber.0.into());
        }

        if widgets.listView.selection().count_selected_rows() == 0 {
            widgets.listView.selection().select_iter(&widgets.listView.model().unwrap().iter_first().unwrap());
        }
        widgets.paned.set_end_child(Some(&gtk::Image::from_pixbuf(Some(&self.solutionSteps[self.activeStep].pixbuf))));
    }
}

#[must_use]
pub fn toRowIndex(rowPath: &gtk::TreePath) -> RowIndex
{
    rowPath.indices()[0].try_into().unwrap()
}

type RowIndex = usize;
