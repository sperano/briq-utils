use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{PathBuf};
use anyhow::{Result};
use clap::{Parser, Subcommand};

mod cache;
mod csv;
mod generator;
mod model;
mod utils;

use utils::pluralize as plrze;

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
        #[arg(short, long)]
        set: Option<String>
    },
    Mirror {
        #[arg(short, long)]
        cache: String,
        #[arg(short, long)]
        workdir: String,
    }
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
                    let data = model::convert(*data);
                    // println!("Generating more Swift code...");
                    // let sets = generator::sets(&data.sets);
                    // fs::write(workdir.join("Sets.swift"), sets)?;
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
        Commands::Analyze { workdir, set } => {
            println!("Reading all CSV data...");
            match csv::read_all(workdir) {
                Ok(data) => {
                    println!("Themes tree has a max depth of {}", get_themes_tree_depth(&data.themes));
                    let materials = data.parts.iter().map(|p|&p.part_material);
                    let materials: HashSet<_> = materials.into_iter().collect();
                    println!("There are {} unique parts materials.", materials.len());

                    println!("Converting data to BRIQ model...");
                    let data = model::convert(*data);
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
                    println!("{} has more than 1 version ({:.1}% of sets). {} has more than 2 versions ({:.1}%).", 
                        plrze(count, "set"), avg+0.5, plrze(count2, "set"), avg2+0.5);
                    if let Some(set) = set {
                        let number = set;
                        let set = data.sets.iter().find(|&s| s.number == *set);
                        if let Some(set) = set {
                            println!("Set {} has {}:", set.number, plrze(set.versions.len(), "version"));
                            let mut all_parts: Vec<Vec<model::SetPart>> = Vec::new();
                            for (i, version) in set.versions.iter().enumerate() {
                                println!("Version #{}: {}, {}.", i, plrze(version.minifigs.len(), "minifig"), plrze(version.parts.len(), "part"));
                                all_parts.push(version.parts.clone());
                                for m in &version.minifigs {
                                    println!("- {:?}", m);
                                }
                            }
                            let mut parts = process(all_parts.clone());
                            let common = parts.pop().unwrap();
                            println!("Parts common to all versions:");
                            for p in &common {
                                println!("- {:?}", p);
                            }
                            for (i, parts) in parts.iter().enumerate() {
                                println!("Parts of version {i}:");
                                for p in parts {
                                    println!("- {:?}", p);
                                }
                            }
                        } else {
                            eprintln!("Invalid set: {}", number)
                        }
                    }
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

fn intersection_all(inputs: &[Vec<model::SetPart>]) -> HashSet<model::SetPart> {
    if inputs.is_empty() {
        return HashSet::new();
    }

    let mut iter = inputs.iter();
    let first: HashSet<_> = iter.next().unwrap().iter().cloned().collect();

    iter.fold(first, |acc, set| {
        acc.intersection(&set.iter().cloned().collect())
            .cloned()
            .collect()
    })
}

fn unique_parts(input: &[model::SetPart], intersection: &HashSet<model::SetPart>) -> Vec<model::SetPart> {
    input
        .iter()
        .filter(|&p| !intersection.contains(p))
        .cloned()
        .collect()
}

fn process(inputs: Vec<Vec<model::SetPart>>) -> Vec<Vec<model::SetPart>> {
    let intersection = intersection_all(&inputs);

    let mut result: Vec<Vec<model::SetPart>> = inputs
        .iter()
        .map(|input| unique_parts(input, &intersection))
        .collect();

    result.push(intersection.into_iter().collect());
    result
}


