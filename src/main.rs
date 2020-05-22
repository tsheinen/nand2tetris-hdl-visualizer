mod error;



use std::{fs, env};
use nand2tetris_hdl_parser::{parse_hdl, Chip, HDLParseError, Part};
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;
use std::path::Path;
use error::GenericError;
use std::fmt::{Display, Formatter};
use subprocess::{Exec, Redirection};
use std::io::{stdout, Write};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Edge {
    source: (String, u32),
    dest: (String, u32),
    pin_name: String,
}

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f,
               "{} -> {} [ label=\" {}\" ];\n",
               format!("{}_{}", self.source.0, self.source.1),
               format!("{}_{}", self.dest.0, self.dest.1),
               self.pin_name
        )
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
        write!(f, "digraph {{\n");
        write!(f, "\tlabel=\"{}\";\n", self.name);
        write!(f, "\tlabelloc=top;\n\tlabeljust=left;\n");
        write!(f, "\t{}_{} [label=\"{}\"];\n", "Input", u32::MAX, "Input");
        write!(f, "\t{}_{} [label=\"{}\"];\n", "Output", u32::MAX, "Output");

        for node in self.nodes.clone() {
            write!(f, "\t{}", node);
        }
        for edge in self.edges.clone() {
            write!(f, "\t{}", edge);
        }
        write!(f, "}}\n")
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
            .map(|(x, y)| (x, y.clone()))
            .collect::<Vec<(usize, Part)>>()
    };
    let mut graph = Graph {
        name: chip.name,
        nodes: parts.iter().map(|(x, y)| Node { name: y.name.clone(), index: x.clone() as u32 }).collect(),
        edges: Vec::new(),
    };
    let mut connections: HashMap<String, (String, u32)> = HashMap::new();
    for (index, part) in parts.clone() {
        for pin in part.external {
            if chip.inputs.iter().any(|x| x.name == pin.name) {
                graph.edges.push(Edge {
                    source: ("Input".to_string(), u32::MAX),
                    dest: (part.name.clone(), index as u32),
                    pin_name: pin.name.clone(),
                })
            }
            if chip.outputs.iter().any(|x| x.name == pin.name) {
                graph.edges.push(Edge {
                    source: (part.name.clone(), index as u32),
                    dest: ("Output".to_string(), u32::MAX),
                    pin_name: pin.name.clone(),
                })
            }
            if connections.contains_key(&pin.name) {
                let conn_chip = resolve(&part.name)?;
                graph.edges.push(if conn_chip.inputs.iter().any(|x| x.name == pin.name) {
                    // current chip has pin in inputs so its the destination chip
                    Edge {
                        source: (part.name.clone(), index as u32),
                        dest: connections.get(&pin.name).unwrap().clone(),
                        pin_name: pin.name,
                    }
                } else {
                    // stored chip has pin in inputs so its the destination chip
                    Edge {
                        source: connections.get(&pin.name).unwrap().clone(),
                        dest: (part.name.clone(), index as u32),
                        pin_name: pin.name,
                    }
                });
            } else {
                connections.insert(pin.name, (part.name.clone(), index as u32));
            }
        }
    }
    Ok(graph)
}

fn main() -> Result<(), GenericError> {
    match which::which("dot") {
        Ok(_) => {},
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

    env::set_current_dir(root_file_path.parent().unwrap());
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
    env::set_current_dir(original_path);
    match matches.value_of("output") {
        Some(T) => fs::write(T, &resp),
        None => stdout().write_all(&resp)
    };


    Ok(())
}