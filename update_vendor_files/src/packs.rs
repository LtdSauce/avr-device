
#[derive(Debug)]
pub struct DownloadablePacks {
    html_page: String,
}

impl DownloadablePacks {
    pub fn from_microchip_website() -> Result<DownloadablePacks, Box<dyn std::error::Error>> {
        let packs_download_microchip_url = "https://packs.download.microchip.com/";
        // TODO improve error handling
        Ok(DownloadablePacks {
            html_page: reqwest::blocking::get(packs_download_microchip_url)?.text()?,
        })
    }
}
