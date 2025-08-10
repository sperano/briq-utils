use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf};
use anyhow::{Result};
use clap::{Parser, Subcommand};
use convert_case::{Case, Casing};

mod csv;
mod model;

#[derive(Parser)]
#[command(name = "briq-utils")]
#[command(about = "Various utilities for BRIQ", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[arg(short, long)]
        workdir: String,
    },
    Validate {
        #[arg(short, long)]
        workdir: String,
    },
    Analyze {
        #[arg(short, long)]
        workdir: String,
    }
}

fn color_csv_to_model(color: csv::ColorRecord) -> model::Color {
    model::Color {
        id: color.id + 1,
        name: color.name,
        rgb: color.rgb,
        num_parts: color.num_parts,
        num_sets: color.num_sets,
        is_transparent: color.is_trans == "True",
        year1: color.y1,
        year2: color.y2,
    }
}

fn theme_csv_to_model(theme: csv::ThemeRecord) -> model::Theme {
    model::Theme {
        id: theme.id,
        name: theme.name,
        parent_id: theme.parent_id,
    }
}

fn part_csv_to_model(part: csv::PartRecord) -> model::Part {
    model::Part {
        number: part.part_num,
        name: part.name,
        part_category_id: part.part_cat_id,
        material: part.part_material,
    }
}

fn minifig_csv_to_model(minifig: csv::MinifigRecord) -> model::Minifig {
    model::Minifig {
        number: minifig.fig_num,
        name: minifig.name,
        parts_count: minifig.num_parts,
        img_url: minifig.img_url,
    }
}

fn set_csv_to_model(set: csv::SetRecord) -> model::Set {
    model::Set {
        number: set.set_num,
        name: set.name,
        year: set.year,
        parts_count: set.num_parts,
        theme_id: set.theme_id,
        img_url: set.img_url,
        versions: vec![],
    }
}

fn get_set_version(inventory_id: u32, version: u16, minifig_inventories: &HashMap<u32, Vec<csv::InventoryMinifigRecord>>, part_inventories: &HashMap<u32, Vec<csv::InventoryPartRecord>>, all_parts_keys: &HashMap<String, bool>) -> model::SetVersion {
    let mut version = model::SetVersion {
        version,
        minifigs: vec![],
        parts: vec![],
    };

    if let Some(minifigs) = minifig_inventories.get(&inventory_id) {
        for minifig in minifigs {
            let minifig = model::SetMinifig {
                number: minifig.fig_num.clone(), // TODO no clone
                quantity: minifig.quantity,
            };
            version.minifigs.push(minifig);
        } 
    }
    if let Some(parts) = part_inventories.get(&inventory_id) {
        for part in parts {
            if all_parts_keys.contains_key(&part.part_num) {
                let part = model::SetPart {
                    number: part.part_num.clone(), // TODO no clone
                    quantity: part.quantity,
                    color_id: part.color_id + 1,
                    img_url: part.img_url.clone(), // TODO no clone
                    is_spare: part.is_spare == "True",
                };
                version.parts.push(part);
            } else {
                eprintln!("Set Version {}: Ignoring part {}: does not exist", version.version, part.part_num);
            }
        }
    }
    version
}

fn convert_to_model(csv_data: csv::Data) -> Box<model::Data> {
    let mut colors: Vec<model::Color> = Vec::with_capacity(csv_data.colors.len());
    for color in csv_data.colors.into_iter() {
        colors.push(color_csv_to_model(color));
    }
    let mut themes: Vec<model::Theme> = Vec::with_capacity(csv_data.themes.len());
    for theme in csv_data.themes.into_iter() {
        themes.push(theme_csv_to_model(theme));
    }
    let mut parts: Vec<model::Part> = Vec::with_capacity(csv_data.parts.len());
    let mut parts_map: HashMap<String, bool> = HashMap::new();
    for part in csv_data.parts.into_iter() {
        parts_map.insert(part.part_num.clone(), true);
        parts.push(part_csv_to_model(part));
    }
    let mut minifigs: Vec<model::Minifig> = Vec::with_capacity(csv_data.minifigs.len());
    for minifig in csv_data.minifigs.into_iter() {
        minifigs.push(minifig_csv_to_model(minifig));
    }
    let mut set_inventories: HashMap<String, Vec<(u32, u16)>> = HashMap::new(); 
    for inventory in csv_data.inventories.into_iter() {
        let versions = set_inventories.entry(inventory.set_num.clone()).or_default();
        versions.push((inventory.id, inventory.version));        
    }
    let mut part_inventories: HashMap<u32, Vec<csv::InventoryPartRecord>> = HashMap::new();
    for part in csv_data.inventories_parts.into_iter() {
        let parts = part_inventories.entry(part.inventory_id).or_default(); 
        parts.push(part);
    }
    let mut minifig_inventories: HashMap<u32, Vec<csv::InventoryMinifigRecord>> = HashMap::new();
    for minifig in csv_data.inventories_minifigs.into_iter() {
        let minifigs = minifig_inventories.entry(minifig.inventory_id).or_default();
        minifigs.push(minifig);
    }
    let mut sets: Vec<model::Set> = Vec::with_capacity(csv_data.sets.len());
    for set in csv_data.sets.into_iter() {
        let mut set = set_csv_to_model(set);
        if let Some(versions) = set_inventories.get(&set.number) {
            for version in versions {
                let version = get_set_version(version.0, version.1, &minifig_inventories, &part_inventories, &parts_map);
                set.versions.push(version);
            }
        }
        sets.push(set);
    }
    Box::new(model::Data{
        colors,
        minifigs,
        parts,
        sets,
        themes,
    })
}

fn generate_swift_code(part_categories: Vec<csv::PartCategoryRecord>) -> String {
    let mut lines = vec![String::from("enum PartCategory: Int {")];
    for cat in part_categories.into_iter() {
        lines.push(format!("   case {} = {}", sanitize_and_case(&cat.name), cat.id))
    }
    lines.push(String::from("}"));
    lines.join("\n")
}

fn sanitize_and_case(s: &str) -> String {
    let replacements = [
        ("&", " and "),
        ("!", " exclam "),
        (",", " "),
        (".", " "),
        ("'", ""),  
        ("\"", ""), 
        ("@", " at "),
        ("#", " number "),
        (":", " "),
        (";", " "),
        ("(", " "),
        (")", " "),
        ("[", " "),
        ("]", " "),
        ("{", " "),
        ("}", " "),
        ("/", " "),
        ("\\", " "),
        ("*", " "),
        ("?", " "),
    ];
    let mut cleaned = s.to_owned();
    for (from, to) in replacements {
        cleaned = cleaned.replace(from, to);
    }
    cleaned.to_case(Case::Camel)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { workdir } => {
            println!("Reading all CSV data...");
            match csv::read_all(workdir) {
                Ok(tup) => {
                    let workdir: PathBuf = workdir.into();
                    println!("Converting data to BRIQ model...");
                    let data = convert_to_model(tup.0);
                    println!("Generating JSON...");
                    let json_string = serde_json::to_string_pretty(&data)?;
                    fs::write(workdir.join("init.json"), json_string)?;
                    println!("Generating Swift code...");
                    let swift_code = generate_swift_code(tup.1);
                    fs::write(workdir.join("PartCategories.swift"), swift_code)?;
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return Err(err)
                }
            }
            Ok(())
        }
        Commands::Validate { workdir } => {
            println!("Reading all CSV data...");
            match csv::read_all(workdir) {
                Ok(tup) => {
                    //let workdir: PathBuf = workdir.into();
                    println!("Validating data...");
                    csv::validate(&tup.0);
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return Err(err)
                }
            }
            Ok(())
        },
        Commands::Analyze { workdir } => {
            println!("Reading all CSV data...");
            match csv::read_all(workdir) {
                Ok(tup) => {
                    println!("Converting data to BRIQ model...");
                    let data = convert_to_model(tup.0);
                    println!("Analyzing data...");
                    let mut count = 0;
                    let mut count2 = 0;
                    for set in &data.sets {
                        if set.versions.len() > 1 {
                            //println!("{} {}: {} versions", set.number, set.name, set.versions.len());
                            count += 1;
                            if set.versions.len() > 2 {
                                count2 += 1
                            }
                        }
                    }
                    let avg = ((count as f32) / (data.sets.len() as f32)) * 100.0;
                    let avg2 = ((count2 as f32) / (data.sets.len() as f32)) * 100.0;
                    println!("{} sets has more than 1 version ({:.1}% of sets). {} sets has more than 2 versions ({:.1}%).", count, avg+0.5, count2, avg2+0.5);
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return Err(err)
                }
            }
            Ok(())
        }
    }    
}
