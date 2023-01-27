use regex::Regex;

#[derive(Debug)]
pub struct DownloadablePacks {
    html_page: String,
}

impl DownloadablePacks {
    const PACKS_URL: &str = "https://packs.download.microchip.com/";

    pub fn from_microchip_website() -> Result<DownloadablePacks, Box<dyn std::error::Error>> {
        // TODO improve error handling
        Ok(DownloadablePacks {
            html_page: reqwest::blocking::get(DownloadablePacks::PACKS_URL)?.text()?
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
                "{}Microchip.{pack_name}.{{version}}.atpack",
                DownloadablePacks::PACKS_URL
            ),
            selected_version: None,
            available_versions: Vec::new(),
        };
        let re = Regex::new(&format!(
            r#"Microchip\.{pack_name}\.(\d*\.\d*\.\d*)\.atpack"#
        ))?;
        for captures in re.captures_iter(&self.html_page) {
            pack.available_versions.push(String::from(&captures[1]))
        }
        Ok(pack)
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct DownloadablePack {
    name: String,
    path_template: String,
    selected_version: Option<String>,
    available_versions: Vec<String>,
}

impl DownloadablePack {
    pub fn version(&mut self, version: &str) -> &DownloadablePack {
        if self.available_versions.contains(&version.to_string()) {
            self.selected_version = Option::from(version.to_string());
        }
        self
    }

    fn url(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.path_template.replace("{version}", self.selected_version.as_ref().unwrap());
        Ok(url)
    }

    fn download(&self) {
        panic!("Not implemented!")
    }
}

#[cfg(test)]
mod tests {
    use crate::packs::DownloadablePacks;

    #[test]
    #[should_panic]
    fn controller_family_panics_if_controller_not_supported() {
        DownloadablePacks { html_page: "BobAndMarry".to_string() }
            .controller_family("hurgel").unwrap();
    }

    #[test]
    fn controller_family_finds_every_version_for_family() {
        let packs = DownloadablePacks { html_page: "BobAndMarry,\nMicrochip.ATmega_DFP.1.atpack//Microchip.ATmega_DFP.1.2.4.atpack".to_string() }
            .controller_family("atmega").unwrap();
        assert_eq!(packs.available_versions, vec!("1.2.4"))
    }

    #[test]
    fn downloadable_pack_returns_download_url_for_selected_version() {
        let url = DownloadablePacks {
            html_page: "BobAndMarry,\nMicrochip.ATmega_DFP.1.1.2.atpack\
        //Microchip.ATmega_DFP.1.2.4.atpack".to_string()
        }.controller_family("atmega").unwrap().version("1.1.2").url();
        assert_eq!(url.unwrap(), "https://packs.download.microchip.com/Microchip.ATmega_DFP.1.1.2.atpack")
    }
}