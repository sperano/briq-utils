use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf};
use anyhow::{Result};
use clap::{Parser, Subcommand};

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
    for part in csv_data.parts.into_iter() {
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
        let mut set = model::Set {
            number: set.set_num,
            name: set.name,
            year: set.year,
            parts_count: set.num_parts,
            theme_id: set.theme_id,
            img_url: set.img_url,
            versions: vec![],
        };
        if let Some(versions) = set_inventories.get(&set.number) {
            for version in versions {
                let inventory_id = version.0;
                let mut version = model::SetVersion {
                    version: version.1,
                    minifigs: vec![],
                    parts: vec![],
                };
                if let Some(minifigs) = minifig_inventories.get(&inventory_id) {
                    for minifig in minifigs {
                        let minifig = model::SetMinifig {
                            number: minifig.fig_num.clone(),
                            quantity: minifig.quantity,
                        };
                        version.minifigs.push(minifig);
                    } 
                }
                if let Some(parts) = part_inventories.get(&inventory_id) {
                    for part in parts {
                        let part = model::SetPart {
                            number: part.part_num.clone(),
                            quantity: part.quantity,
                            color_id: part.color_id,
                            img_url: part.img_url.clone(),
                            is_spare: part.is_spare == "True",
                        };
                        version.parts.push(part);
                    }
                }
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
    String::from("foo")
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
    }    
}
