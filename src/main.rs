use std::{path::Path, process};

use crate::tui::termsh_run;
use clap::Parser;

mod gmsh_ctl;
mod tui;

fn main() {
    //use clap to read in the step file name
    let args = CliArgs::parse();

    //validate filename
    //Check whether .step file name is legal and exist
    if !args.step_file.contains(".step") {
        println!("please input a .step file");
        process::exit(1);
    }

    if !Path::new(&args.step_file).exists() {
        println!("file {} does not exist", args.step_file);
        process::exit(1);
    }

    if let Err(e) = termsh_run(args.step_file) {
        panic!("{}", e)
    }

    //next: save a termsh_cache.json to store Physical Name and Physical IDs in current folder
    //and when start, load these Physical Names and Physical IDs
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long)]
    #[arg(index = 1)]
    step_file: String,
}
