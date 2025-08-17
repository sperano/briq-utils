
#[derive(Debug, serde::Serialize)]
pub struct Color {
    pub id: i32,
    pub name: String,
    pub rgb: String,
    pub is_transparent: bool,
    pub num_parts: u32,
    pub num_sets: u32,
    pub year1: Option<u16>,
    pub year2: Option<u16>,
}

#[derive(Debug, serde::Serialize)]
pub struct Minifig {
    pub number: String,
    pub name: String,
    pub parts_count: u32,
    pub img_url: String,
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
    pub img_url: String,
    pub versions: Vec<SetVersion>,
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
    pub color_id: i32,
    pub quantity: u16,
    pub is_spare: bool,
    pub img_url: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Theme {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
}

#[derive(Debug, serde::Serialize)]
pub struct Data {
    //pub colors: Vec<Color>,
    //color_ids: HashMap<i32, usize>,
    pub minifigs: Vec<Minifig>,
    pub parts: Vec<Part>,
    pub sets: Vec<Set>,
    pub themes: Vec<Theme>,
}

// impl Data {
//     fn load_color_ids_map(&mut self) {
//         for (i, color) in self.colors.iter().enumerate() {
//             self.color_ids.insert(color.id, i);
//         }
//     }  
// }
//
