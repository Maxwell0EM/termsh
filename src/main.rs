use crate::tui::tui_start;

mod gmsh_ctl;
mod tui;

fn main() {
    //use clap to read in the step file name

    let filename = "test.step";

    let _ = tui_start(String::from(filename));
}
