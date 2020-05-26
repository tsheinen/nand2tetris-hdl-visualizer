//! visualization utility for nand2tetris HDL

#![forbid(unsafe_code)]
#![deny(
missing_debug_implementations,
missing_docs,
trivial_casts,
trivial_numeric_casts,
unused_extern_crates,
unused_import_braces,
unused_qualifications,
unused_results,
warnings
)]

mod error;


use std::{fs, env};
use nand2tetris_hdl_parser::{parse_hdl, Chip, Part};
use std::collections::{HashMap, BTreeSet};
use std::path::Path;
use error::GenericError;
use std::fmt::{Display, Formatter};
use subprocess::{Exec, Redirection};
use std::io::{stdout, Write};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Edge {
    source: BTreeSet<(String, u32)>,
    dest: BTreeSet<(String, u32)>,
    pin_name: String,
}

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for (source_pin, source_index) in &self.source {
            for (dest_pin, dest_index) in &self.dest {
                write!(f,
                       "{}_{} -> {}_{} [ label=\" {}\" ];\n", source_pin, source_index, dest_pin, dest_index, self.pin_name)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Node {
    name: String,
    index: u32,
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f,
               "{}_{} [ label=\" {}\" ];\n",
               self.name,
               self.index,
               self.name
        )
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Graph {
    name: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Display for Graph {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "digraph {{\n")?;
        write!(f, "\tlabel=\"{}\";\n", self.name)?;
        write!(f, "\tlabelloc=top;\n\tlabeljust=left;\n")?;
        write!(f, "\t{}_{} [label=\"{}\"];\n", "Input", u32::MAX, "Input")?;
        write!(f, "\t{}_{} [label=\"{}\"];\n", "Output", u32::MAX, "Output")?;

        for node in &self.nodes {
            write!(f, "\t{}", node)?;
        }
        for edge in &self.edges {
            write!(f, "\t{}", edge)?;
        }
        write!(f, "}}\n")?;
        Ok(())
    }
}


fn resolve(chip: &str) -> Result<Chip, GenericError> {
    let filename = &[&chip.split("_").next().unwrap(), ".hdl"].join("");
    let hdl = fs::read_to_string(filename)?;
    Ok(parse_hdl(&hdl)?)
}

fn generate_graph(filename: &str) -> Result<Graph, GenericError> {
    let chip = resolve(filename)?;
    let parts = {
        chip.parts
            .iter()
            .enumerate()
            .map(|(x, y)| (x, y))
            .collect::<Vec<(usize, &Part)>>()
    };

    let mut connections = HashMap::<String, Edge>::new();
    for (index, part) in &parts {
        for (internal, external) in part.internal.iter().zip(&part.external) {
            let part_chip = resolve(&part.name)?;
            let any_input = part_chip.inputs.iter().any(|x| x == internal);
            let any_output = part_chip.outputs.iter().any(|x| x == internal);
            let any_chip_input = chip.inputs.iter().any(|x| x == external);
            let any_chip_output = chip.outputs.iter().any(|x| x == external);
            if !connections.contains_key(&external.name) {
                let _ = connections.insert(external.name.clone(), Edge {
                    source: BTreeSet::new(),
                    dest: BTreeSet::new(),
                    pin_name: external.name.clone(),
                });
            }
            let edge = match connections.get_mut(&external.name) {
                Some(edge) => edge,
                None => continue
            };

            if (!any_input && !any_output) || (any_input && any_output) {
                panic!("Unexpected behaviour is occurring, please report this :)");
            } else if any_input {
                let _ = edge.dest.insert((part.name.clone(), *index as u32));
            } else if any_output {
                let _ = edge.source.insert((part.name.clone(), *index as u32));
            }
            if any_chip_input && !edge.source.contains(&(String::from("Input"), u32::MAX)) {
                let _ = edge.source.insert((String::from("Input"), u32::MAX));
            } else if any_chip_output && !edge.source.contains(&(String::from("Input"), u32::MAX)) {
                let _ = edge.dest.insert((String::from("Output"), u32::MAX));
            }
        }
    }
    Ok(Graph {
        name: chip.name,
        nodes: parts.iter().map(|(x, y)| Node { name: y.name.clone(), index: x.clone() as u32 }).collect(),
        edges: connections.iter().map(|(_, y)| y.clone()).collect(),
    })
}

fn main() -> Result<(), GenericError> {
    match which::which("dot") {
        Ok(_) => {}
        Err(_) => {
            eprintln!("This utility requires GraphViz to be installed and in your PATH");
            std::process::exit(1)
        }
    }

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("output")
            .long("output")
            .short("o")
            .value_name("OUTPUT")
            .takes_value(true)
            .help("File to write output to. Uses stdout by default"))
        .arg(Arg::with_name("recursive")
            .long("recursive")
            .short("r")
            .help("Recursively graph children gates - NOT IMPLEMENTED YET"))
        .arg(Arg::with_name("FILE")
            .help("Sets the input HDL file to use")
            .required(true)
            .index(1))
        .get_matches();

    let root_file = matches.value_of("FILE").expect("Missing file - a file is required by clap so this should never happen");
    let root_file_path = Path::new(root_file);

    let original_path = env::current_dir()?;

    env::set_current_dir(root_file_path.parent().unwrap())?;
    let root_chip_name = root_file_path.file_name().unwrap().to_str().unwrap().split(".").next().unwrap();
    let graph = generate_graph(root_chip_name)?;
    let resp = Exec::cmd("dot")
        .arg("-Tpng")
        .stdin(format!("{}", graph).as_str())
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture()
        .unwrap()
        .stdout;
    env::set_current_dir(original_path)?;
    match matches.value_of("output") {
        Some(t) => fs::write(t, &resp)?,
        None => stdout().write_all(&resp)?
    };


    Ok(())
}