
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

pub fn mirror(url: &str, cache_dir: &str) -> Result<PathBuf> {
    if let Some((_, path)) = url.split_once("//") {
        // return early if path already exists, make sure the parent directories exists
        let path = Path::new(cache_dir).join(path);
        if path.exists() {
            println!("{} already exists.", path.display());
            return Ok(path);
        }
        fs::create_dir_all(path.parent().unwrap())?;
       
        // simple blocking download of url into path
        let mut resp = reqwest::blocking::get(url)?;
        if !resp.status().is_success() {
            return Err(anyhow!("Request failed: {}", resp.status()));
        }
        let mut out = fs::File::create(&path)?;
        io::copy(&mut resp, &mut out)?;
        println!("{} downloaded.", url);        
        Ok(path)
    } else {
        Err(anyhow!("invalid URL: {}", url))
    }
}
