mod error;

#[macro_use]
extern crate cached;

use std::{fs, env};
use nand2tetris_hdl_parser::{parse_hdl, Chip, HDLParseError, Part};
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;
use std::path::Path;
use error::GenericError;

struct Edge {
    source: String,
    dest: String,
    pin_name: String,
}

fn resolve(chip: &str) -> Result<Chip, GenericError> {
    let filename = &[&chip.split("_").next().unwrap(), ".hdl"].join("");
    let hdl = fs::read_to_string(filename)?;
    Ok(parse_hdl(&hdl)?)
}

fn generate_graph(filename: &str) -> Result<Vec<Edge>, GenericError> {
    let chip = resolve(filename)?;
    let parts = {
        let mut occurrences: HashMap<String, u32> = HashMap::new();
        chip.parts
            .iter()
            .enumerate()
            .map(|(x, y)| {
                let new_name = {
                    if occurrences.contains_key(&y.name) {
                        let count = occurrences.entry(y.name.clone()).or_insert(1);
                        *count += 1;
                        String::from("_") + &(*count).to_string()
                    } else {
                        occurrences.entry(y.name.clone()).or_insert(1);
                        String::from("")
                    }
                };
                Part {
                    name: y.clone().name + &new_name,
                    ..y.clone()
                }
            })
            .collect::<Vec<(Part)>>()
    };
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
                let conn_chip = resolve(&part.name)?;
                graph.push(if conn_chip.inputs.iter().any(|x| x.name == pin.name) {
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
                });
            } else {
                connections.insert(pin.name, part.name.clone());
            }
        }
    }
    Ok(graph)
}

fn main() -> Result<(), GenericError> {
    let root_file = &env::args().nth(1).unwrap();
    let root_file_path = Path::new(root_file);
    env::set_current_dir(root_file_path.parent().unwrap());
    let root_chip_name = root_file_path.file_name().unwrap().to_str().unwrap().split(".").next().unwrap();
    let v = generate_graph(root_chip_name)?;
    println!("digraph {{");
    for edge in v {
        println!("\t{} -> {} [ label=\" {}\" ];", edge.source, edge.dest, edge.pin_name);
    }
    println!("}}");
    Ok(())
}