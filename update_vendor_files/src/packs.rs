use regex::Regex;

#[derive(Debug)]
pub struct DownloadablePacks {
    html_page: String,
    packs_url: String,
}

impl DownloadablePacks {
    pub fn from_microchip_website() -> Result<DownloadablePacks, Box<dyn std::error::Error>> {
        let packs_download_microchip_url = "https://packs.download.microchip.com/";
        // TODO improve error handling
        Ok(DownloadablePacks {
            html_page: reqwest::blocking::get(packs_download_microchip_url)?.text()?,
            packs_url: packs_download_microchip_url.to_string(),
        })
    }

    pub fn controller_family(
        &self,
        name: &str,
    ) -> Result<DownloadablePack, Box<dyn std::error::Error>> {
        // The whole project uses the lowercase representation, so we do that too and convert it here
        // Additionally, those packs have some suffixes which are applied here
        let pack_name = match name {
            "atmega" => "ATmega_DFP",
            _ => panic!("{name} is not a valid controller family!"),
        };

        let mut pack = DownloadablePack {
            name: name.to_string(),
            path_template: format!(
                "{}/Microchip.{pack_name}.{{version}}.atpack",
                self.packs_url
            ),
            selected_version: None,
            available_versions: Vec::new(),
        };
        let re = Regex::new(&format!(
            r#"Microchip\.{pack_name}\.(\d.*\d.*\d.*)\.atpack"#
        ))?;
        for captures in re.captures_iter(&self.html_page) {
            pack.available_versions.push(String::from(&captures[1]))
        }
        Ok(pack)
    }
}

#[derive(Debug)]
pub struct DownloadablePack {
    name: String,
    path_template: String,
    selected_version: Option<String>,
    available_versions: Vec<String>,
}

#[cfg(test)]
mod tests {
    // TODO write tests!
}
