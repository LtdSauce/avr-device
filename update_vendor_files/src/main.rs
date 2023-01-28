mod packs;

use std::env;

fn parse_args(args: Vec<String>) -> packs::PackInfo {
    // ToDo emit error message and usage string
    packs::PackInfo::from_str(&args[1])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pack = parse_args(env::args().collect());
    let pack = packs::DownloadablePacks::from_microchip_website()?.for_pack(&pack)?;
    println!("{pack:?}");
    Ok(())
}


