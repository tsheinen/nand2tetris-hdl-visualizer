#[macro_use]
extern crate cached;

use std::{fs, env};
use nand2tetris_hdl_parser::{parse_hdl, Chip, HDLParseError, Part};
use std::collections::{HashSet, HashMap};
use std::io::Error;
use std::iter::FromIterator;
use std::path::Path;


struct Edge {
    source: String,
    dest: String,
    pin_name: String,
}

fn resolve(chip: &str) -> Chip {
    let filename = &[&chip.split("_").next().unwrap(), ".hdl"].join("");
    let hdl = match fs::read_to_string(filename) {
        Ok(f) => f,
        Err(e) => panic!("Could not read file {:?}: {:?}", filename, e),
    };
    parse_hdl(&hdl).unwrap()
}

fn generate_graph(filename: &str) -> Vec<Edge> {
    let chip = resolve(filename);
    let mut occurences: HashMap<String, u32> = HashMap::new();
    let parts = chip.parts
        .iter()
        .enumerate()
        .map(|(x, y)| {
            let new_name = {
                if occurences.contains_key(&y.name) {
                    let count = occurences.entry(y.name.clone()).or_insert(1);
                    *count += 1;
                    String::from("_") + &(*count).to_string()
                } else {
                    occurences.entry(y.name.clone()).or_insert(1);
                    String::from("")
                }
            };
            Part {
                name: y.clone().name + &new_name,
                ..y.clone()
            }
        })
        .collect::<Vec<(Part)>>();
    let mut graph: Vec<Edge> = Vec::new();
    let mut connections: HashMap<String, String> = HashMap::new();
    for part in parts {
        for pin in part.external {
            if chip.inputs.iter().any(|x| x.name == pin.name) {
                graph.push(Edge {
                    source: "Input".to_string(),
                    dest: part.name.clone(),
                    pin_name: pin.name.clone(),
                })
            }
            if chip.outputs.iter().any(|x| x.name == pin.name) {
                graph.push(Edge {
                    source: part.name.clone(),
                    dest: "Output".to_string(),
                    pin_name: pin.name.clone(),
                })
            }
            if connections.contains_key(&pin.name) {
                let conn_chip = resolve(&part.name);
                let edge = if conn_chip.inputs.iter().any(|x| x.name == pin.name) {
                    // current chip has pin in inputs so its the destination chip
                    Edge {
                        source: part.name.clone(),
                        dest: connections.get(&pin.name).unwrap().clone(),
                        pin_name: pin.name,
                    }
                } else {
                    // stored chip has pin in inputs so its the destination chip
                    Edge {
                        source: connections.get(&pin.name).unwrap().clone(),
                        dest: part.name.clone(),
                        pin_name: pin.name,
                    }
                };
                graph.push(edge)
            } else {
                connections.insert(pin.name, part.name.clone());
            }
        }
    }
    graph
}

fn main() -> Result<(), Error> {
    let path = &env::args().nth(1).unwrap();
    let file = Path::new(path);
    env::set_current_dir(file.parent().unwrap());
    let mut root_chip = file.file_name().unwrap().to_str().unwrap().split(".").next().unwrap();
    let v = generate_graph(root_chip);
    println!("digraph {{");
    for edge in v {
        println!("\t{} -> {} [ label=\" {}\" ];", edge.source, edge.dest, edge.pin_name);
    }
    println!("}}");
    Ok(())
}