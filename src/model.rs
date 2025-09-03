use std::collections::{HashMap};
use phf::phf_map;
use serde::ser::{Serialize, Serializer, SerializeStruct};
use crate::utils::convert_asset_url;
use crate::csv::{InventoryPartRecord, InventoryMinifigRecord, MinifigRecord, PartRecord, SetRecord};
use crate::csv::Data as CSVData;

#[derive(Debug, serde::Serialize)]
pub struct Minifig {
    pub number: String,
    pub name: String,
    pub parts_count: u32,
    pub img_url: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct Part {
    pub number: String,
    pub name: String,
    pub part_category_id: u32,
    pub material: String,
}

#[derive(Debug)]
pub struct Set {
    pub number: String,
    pub name: String,
    pub year: u16,
    pub theme_id: u32,
    pub parts_count: u32, // ? relevant ?? doesn't vary per versions? TODO
    pub img_url: Option<String>,
    pub is_pack: bool,
    pub is_unreleased: bool,
    pub is_accessories: bool,
    pub versions: Vec<SetVersion>, 
} 

impl Set {
    fn minifigs(&self) -> &Vec<SetMinifig> {
        &self.versions.last().unwrap().minifigs
    }
    fn parts(&self) -> &Vec<SetPart> {
        &self.versions.last().unwrap().parts
    }
}

impl Serialize for Set {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where 
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Set", 2)?;
        state.serialize_field("number", &self.number)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("year", &self.year)?;
        state.serialize_field("theme_id", &self.theme_id)?;
        state.serialize_field("parts_count", &self.parts_count)?;
        state.serialize_field("img_url", &self.img_url)?;
        state.serialize_field("minifigs", &self.minifigs())?;
        state.serialize_field("parts", &self.parts())?;
        state.serialize_field("is_pack", &self.is_pack)?;
        state.serialize_field("is_unreleased", &self.is_unreleased)?;
        state.serialize_field("is_accessories", &self.is_accessories)?;
        state.end()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SetVersion {
    pub version: u16,
    pub minifigs: Vec<SetMinifig>,
    pub parts: Vec<SetPart>,
} 

#[derive(Debug, serde::Serialize)]
pub struct SetMinifig {
    pub number: String,
    pub quantity: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct SetPart {
    pub number: String,
    pub color_id: u32,
    pub quantity: u16,
    pub is_spare: bool,
    pub img_url: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct Data {
    pub minifigs: Vec<Minifig>,
    pub parts: Vec<Part>,
    pub sets: Vec<Set>,
}

fn get_set_version(inventory_id: u32, version: u16, minifig_inventories: &HashMap<u32, Vec<InventoryMinifigRecord>>, part_inventories: &HashMap<u32, Vec<InventoryPartRecord>>, all_parts_keys: &HashMap<String, bool>) -> SetVersion {
    let mut version = SetVersion {
        version,
        minifigs: vec![],
        parts: vec![],
    };

    if let Some(minifigs) = minifig_inventories.get(&inventory_id) {
        for minifig in minifigs {
            let minifig = SetMinifig {
                number: minifig.fig_num.clone(), // TODO no clone
                quantity: minifig.quantity,
            };
            version.minifigs.push(minifig);
        } 
    }
    if let Some(parts) = part_inventories.get(&inventory_id) {
        for part in parts {
            if all_parts_keys.contains_key(&part.part_num) {
                let part = SetPart {
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

pub fn convert(csv_data: CSVData) -> Box<Data> {
    let mut parts: Vec<Part> = Vec::with_capacity(csv_data.parts.len());
    let mut parts_map: HashMap<String, bool> = HashMap::new();
    for part in csv_data.parts.into_iter() {
        parts_map.insert(part.part_num.clone(), true);
        parts.push(part_csv_to_model(part));
    }
    let mut minifigs: Vec<Minifig> = Vec::with_capacity(csv_data.minifigs.len());
    for minifig in csv_data.minifigs.into_iter() {
        minifigs.push(minifig_csv_to_model(minifig));
    }
    let mut set_inventories: HashMap<String, Vec<(u32, u16)>> = HashMap::new(); 
    for inventory in csv_data.inventories.into_iter() {
        let versions = set_inventories.entry(inventory.set_num.clone()).or_default();
        versions.push((inventory.id, inventory.version));        
    }
    let mut part_inventories: HashMap<u32, Vec<InventoryPartRecord>> = HashMap::new();
    for part in csv_data.inventories_parts.into_iter() {
        let parts = part_inventories.entry(part.inventory_id).or_default(); 
        parts.push(part);
    }
    let mut minifig_inventories: HashMap<u32, Vec<InventoryMinifigRecord>> = HashMap::new();
    for minifig in csv_data.inventories_minifigs.into_iter() {
        let minifigs = minifig_inventories.entry(minifig.inventory_id).or_default();
        minifigs.push(minifig);
    }
    let mut sets: Vec<Set> = Vec::with_capacity(csv_data.sets.len());
    for set in csv_data.sets.into_iter() {
        let key = set.set_num.clone();
        let mut set = set_csv_to_model(set, is_pack(&key), is_unreleased(&key), is_accessories(&key));
        if let Some(versions) = set_inventories.get(&set.number) {
            for version in versions {
                let version = get_set_version(version.0, version.1, &minifig_inventories, &part_inventories, &parts_map);
                set.versions.push(version);
            }
        }
        sets.push(set);
    }
    Box::new(Data{
        minifigs,
        parts,
        sets,
    })
}

fn part_csv_to_model(part: PartRecord) -> Part {
    Part {
        number: part.part_num,
        name: part.name,
        part_category_id: part.part_cat_id,
        material: part.part_material,
    }
}

fn minifig_csv_to_model(minifig: MinifigRecord) -> Minifig {
    Minifig {
        number: minifig.fig_num,
        name: minifig.name,
        parts_count: minifig.num_parts,
        img_url: convert_asset_url(&minifig.img_url),
}
}

fn set_csv_to_model(set: SetRecord, is_pack: bool, is_unreleased: bool, is_accessories: bool) -> Set {
    Set {
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

static PACKS: phf::Map<&'static str, ()> = phf_map! {
    "1507-1" => (),
    "1510-1" => (),
    "1616-1" => (),
    "1969-2" => (),
    "1977-1" => (),
    "1983-1" => (),
    "1999-1" => (),
};

pub fn is_pack(key: &str) -> bool {
    PACKS.contains_key(key) 
}

static UNRELEASED: phf::Map<&'static str, ()> = phf_map! {
    "1526-1" => (),
};

pub fn is_unreleased(key: &str) -> bool {
    UNRELEASED.contains_key(key)
}

static ACCESSORIES: phf::Map<&'static str, ()> = phf_map! {
   "6921-1" => (), 
};

pub fn is_accessories(key: &str) -> bool {
    ACCESSORIES.contains_key(key)
}
