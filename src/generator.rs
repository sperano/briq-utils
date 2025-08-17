use convert_case::{Case, Casing};
use crate::csv::{ColorRecord, PartCategoryRecord};

pub fn generate_part_categories(part_categories: Vec<PartCategoryRecord>) -> String {
    let mut lines = vec![String::from("enum PartCategory: Int {")];
    for cat in part_categories.into_iter() {
        lines.push(format!("   case {} = {}", sanitize_and_case(&cat.name), cat.id))
    }
    lines.push(String::from("}"));
    lines.join("\n")
}

pub fn generate_colors(colors: Vec<ColorRecord>) -> String {
    let mut lines = vec![String::from("")];

    lines.join("\n")
}

pub fn generate_swift_code(part_categories: Vec<PartCategoryRecord>, colors: Vec<ColorRecord>) -> (String, String) {
    (generate_part_categories(part_categories), generate_colors(colors))
}

pub fn sanitize_and_case(s: &str) -> String {
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


