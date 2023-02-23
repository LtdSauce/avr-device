mod packs;

use std::env;

fn parse_args(mut args: env::Args) -> packs::PackInfo {
    // ToDo emit error message and usage string
    packs::PackInfo::from_str(args.nth(1).as_ref().unwrap())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pack = parse_args(env::args());
    let pack = packs::DownloadablePacks::from_microchip_website()?.for_pack(pack)?;
    println!("{pack:?}");
    println!("{:?}",pack.download()?.list_controllers()?);
    Ok(())
}
