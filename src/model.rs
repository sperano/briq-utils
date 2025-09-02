use phf::phf_map;

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

#[derive(Debug, serde::Serialize)]
pub struct Set {
    pub number: String,
    pub name: String,
    pub year: u16,
    pub theme_id: u32,
    pub parts_count: u32, // ? relevant ?? doesn't vary per versions? TODO
    pub img_url: Option<String>,
    pub versions: Vec<SetVersion>,
    pub is_pack: bool,
    pub is_unreleased: bool,
    pub is_accessories: bool,
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

#[derive(Debug, serde::Serialize)]
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
