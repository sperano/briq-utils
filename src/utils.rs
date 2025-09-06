use inflector::string::pluralize::to_plural;

pub fn pluralize(count: usize, word: &str) -> String {
    if count == 1 {
        format!("1 {word}")
    } else {
        format!("{} {}", count, to_plural(word))
    }
}

pub fn convert_asset_url(url: &str) -> Option<String> {
    if !url.is_empty() {
        static PREFIX: &str = "https://cdn.rebrickable.com/media";
        return Some(format!("https://briq-assets.spe.quebec{}", &url[PREFIX.len()..]));
    }
    None
}

