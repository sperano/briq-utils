use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf};
use anyhow::{Result};
use clap::{Parser, Subcommand};

mod cache;
mod csv;
mod generator;
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
    },
    Mirror {
        #[arg(short, long)]
        cache: String,
        #[arg(short, long)]
        workdir: String,
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
        img_url: convert_asset_url(&minifig.img_url),
    }
}

fn set_csv_to_model(set: csv::SetRecord, is_pack: bool, is_unreleased: bool, is_accessories: bool) -> model::Set {
    model::Set {
        number: set.set_num,
        name: set.name,
        year: set.year,
        parts_count: set.num_parts,
        theme_id: set.theme_id,
        img_url: convert_asset_url(&set.img_url),
        versions: vec![],
        is_pack,
        is_unreleased,
        is_accessories,
    }
}

fn convert_asset_url(url: &str) -> Option<String> {
    if !url.is_empty() {
        static PREFIX: &str = "https://cdn.rebrickable.com/media";
        return Some(format!("https://briq-assets.spe.quebec{}", &url[PREFIX.len()..]));
    }
    None
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
                    color_id: part.color_id.try_into().unwrap(),
                    img_url: convert_asset_url(&part.img_url),
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
        let key = set.set_num.clone();
        let mut set = set_csv_to_model(set, model::is_pack(&key), model::is_unreleased(&key), model::is_accessories(&key));
        if let Some(versions) = set_inventories.get(&set.number) {
            for version in versions {
                let version = get_set_version(version.0, version.1, &minifig_inventories, &part_inventories, &parts_map);
                set.versions.push(version);
            }
        }
        sets.push(set);
    }
    Box::new(model::Data{
        minifigs,
        parts,
        sets,
        //themes,
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { workdir } => {
            println!("Reading all CSV data...");
            match csv::read_all(workdir) {
                Ok(data) => {
                    let workdir: PathBuf = workdir.into();
                    println!("Generating Swift code...");
                    let part_cats = generator::part_categories(&data.part_categories);
                    fs::write(workdir.join("PartCategories.swift"), part_cats)?;
                    let part_colors = generator::colors(&data.colors);
                    fs::write(workdir.join("PartColors.swift"), part_colors)?;
                    let themes = generator::themes(&data.themes);
                    fs::write(workdir.join("Themes.swift"), themes)?;
                    println!("Converting data to BRIQ model...");
                    let data = convert_to_model(*data);
                    println!("Generating JSON...");
                    let json_string = serde_json::to_string_pretty(&data)?;
                    fs::write(workdir.join("init.json"), json_string)?;
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
                Ok(data) => {
                    //let workdir: PathBuf = workdir.into();
                    println!("Validating data...");
                    csv::validate(&data);
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
                Ok(data) => {
                    println!("Themes tree has a max depth of {}", get_themes_tree_depth(&data.themes));
                    println!("Converting data to BRIQ model...");
                    let data = convert_to_model(*data);
                    println!("Analyzing data...");
                    let mut count = 0;
                    let mut count2 = 0;
                    for set in &data.sets {
                        if set.versions.len() > 1 {
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
        },
        Commands::Mirror { cache, workdir } => {
            println!("Reading all CSV data...");
            match csv::read_all(workdir) {
                Ok(data) => {
                    let mut urls: Vec<String> = data.inventories_parts.iter()
                        .filter(|p| !p.img_url.is_empty())
                        .map(|p| p.img_url.clone()).collect();
                    urls.extend(data.sets.iter()
                        .filter(|s| !s.img_url.is_empty())
                        .map(|s| s.img_url.clone()));
                    urls.extend(data.minifigs.iter()
                        .filter(|m| !m.img_url.is_empty())
                        .map(|m| m.img_url.clone()));
                    urls.sort();
                    urls.dedup();
                    let total = urls.len();
                    let mut i = 0;
                    for url in urls {
                        i += 1;
                        print!("{:.2}% ", (i as f64 / total as f64) * 100.0);
                        if let Err(err) = cache::mirror(&url, cache) {
                            eprintln!("\x1b[31m{} {}\x1b[0m", url, err);
                        }
                    } 
                    Ok(())
                }
                Err(err) => {
                    eprintln!("{}", err);
                    Err(err)
                }
            }
        }
    }    
}

fn get_themes_tree_depth(themes: &[csv::ThemeRecord]) -> u32 {
    let mut m: HashMap<u32, &csv::ThemeRecord> = HashMap::new();
    for theme in themes {
        m.insert(theme.id, theme);
    }
    let mut max = 0;
    for theme in themes {
        let mut count = 0;
        let mut th = theme;
        loop {
            count += 1;
            if th.parent_id.is_none() {
                break
            }
            th = m[&th.parent_id.unwrap()]
        }
        if count > max {
            max = count
        }
    };
    max
}
