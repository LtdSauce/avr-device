mod packs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pack = packs::DownloadablePacks::from_microchip_website()?.controller_family("atmega")?;
    println!("{pack:?}");
    Ok(())
}
