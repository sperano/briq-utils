use std::fs::File;
use std::path::PathBuf;
use anyhow::{Result};
use csv::Reader;

macro_rules! make_csv_reader {
    ($fn_name:ident, $type:ty) => {
        fn $fn_name(path: PathBuf) -> anyhow::Result<Vec<$type>> {
            let mut records = Vec::<$type>::new();
            let mut rdr = Reader::from_reader(File::open(&path)?);
            for result in rdr.deserialize() {
                let record: $type = result?;
                records.push(record);
            }
            Ok(records)
        }
    };
}

macro_rules! read_csv {
    ($workdir:expr, $file:literal, $func:ident) => {
        $func($workdir.join($file))?
    };
}

#[derive(Debug, serde::Deserialize)]
pub struct ColorRecord {
    pub id: i32,
    pub name: String,
    pub rgb: String,
    pub is_trans: String,
    pub num_parts: u32,
    pub num_sets: u32,
    pub y1: Option<u16>,
    pub y2: Option<u16>,
}

#[derive(Debug, serde::Deserialize)]
pub struct InventoryRecord {
    pub id: u32,
    pub version: u16,
    pub set_num: String
}

#[derive(Debug, serde::Deserialize)]
pub struct InventoryMinifigRecord {
    pub inventory_id: u32,
    pub fig_num: String,
    pub quantity: u16,
}

#[derive(Debug, serde::Deserialize)]
pub struct InventoryPartRecord {
    pub inventory_id: u32,
    pub part_num: String,
    pub color_id: i32,
    pub quantity: u16,
    pub is_spare: String,
    pub img_url: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct MinifigRecord {
    pub fig_num: String,
    pub name: String,
    pub num_parts: u32,
    pub img_url: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct PartRecord {
    pub part_num: String,
    pub name: String,
    pub part_cat_id: u32,
    pub part_material: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct PartCategoryRecord {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SetRecord {
    pub set_num: String,
    pub name: String,
    pub year: u16,
    pub theme_id: u32,
    pub num_parts: u32,
    pub img_url: String,
} 

#[derive(Debug, serde::Deserialize)]
pub struct ThemeRecord {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
}

#[derive(Debug)]
pub struct Data {
    pub colors: Vec<ColorRecord>,
    pub inventories: Vec<InventoryRecord>,
    pub inventories_minifigs: Vec<InventoryMinifigRecord>,
    pub inventories_parts: Vec<InventoryPartRecord>,
    pub minifigs: Vec<MinifigRecord>,
    pub parts: Vec<PartRecord>,
    pub sets: Vec<SetRecord>,
    pub themes: Vec<ThemeRecord>,
}

make_csv_reader!(read_colors, ColorRecord);
make_csv_reader!(read_inventories, InventoryRecord);
make_csv_reader!(read_inventories_minifigs, InventoryMinifigRecord);
make_csv_reader!(read_inventories_parts, InventoryPartRecord);
make_csv_reader!(read_minifigs, MinifigRecord);
make_csv_reader!(read_part_categories, PartCategoryRecord);
make_csv_reader!(read_parts, PartRecord);
make_csv_reader!(read_sets, SetRecord);
make_csv_reader!(read_themes, ThemeRecord);

pub fn read_all(workdir: &str) -> Result<Box<(Data, Vec<PartCategoryRecord>)>> {
    let workdir: PathBuf = workdir.into();
    let data = Data{
        colors: read_csv!(workdir, "colors.csv", read_colors),
        inventories: read_csv!(workdir, "inventories.csv", read_inventories),
        inventories_minifigs: read_csv!(workdir, "inventory-minifigs.csv", read_inventories_minifigs),
        inventories_parts: read_csv!(workdir, "inventory-parts.csv", read_inventories_parts),
        minifigs: read_csv!(workdir, "minifigs.csv", read_minifigs),
        parts: read_csv!(workdir, "parts.csv", read_parts),
        sets: read_csv!(workdir, "sets.csv", read_sets),
        themes: read_csv!(workdir, "themes.csv", read_themes),
    };
    let part_categories = read_csv!(workdir, "part-categories.csv", read_part_categories);
    Ok(Box::new((data, part_categories)))
}

