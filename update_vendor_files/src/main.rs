use regex::Regex;

mod packs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", packs::DownloadablePacks::from_microchip_website());
    Ok(())
}
