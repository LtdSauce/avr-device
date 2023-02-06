use regex::Regex;

// TODO improve error handling: at least swap panics with result errors?

#[derive(PartialEq, Debug)]
pub struct PackInfo {
    pub name: String,
    pub selected_version: Option<String>,
}

impl PackInfo {
    pub fn from_str(arg: &str) -> PackInfo {
        match arg.split_once("==") {
            Some((name, version)) => Self::from(name, version),
            None => Self::from_name(arg)
        }
    }

    fn from(name: &str, version: &str) -> PackInfo {
        PackInfo {
            name: name.to_string(),
            selected_version: Some(version.to_string()),
        }
    }

    fn from_name(name: &str) -> PackInfo {
        PackInfo {
            name: name.to_string(),
            selected_version: None,
        }
    }
}

#[derive(Debug)]
pub struct DownloadablePacks {
    html_page: String,
}

impl DownloadablePacks {
    const PACKS_URL: &str = "https://packs.download.microchip.com/";

    pub fn from_microchip_website() -> Result<DownloadablePacks, reqwest::Error> {
        Ok(DownloadablePacks {
            html_page: reqwest::blocking::get(DownloadablePacks::PACKS_URL)?.text()?,
        })
    }

    pub fn for_pack(
        &self,
        pack: &PackInfo,
    ) -> Result<DownloadablePack, Box<dyn std::error::Error>> {
        // The whole project uses the lowercase representation, so we do that too and convert it here
        // Additionally, those packs have some suffixes which are applied here
        let pack_name = match pack.name.as_str() {
            "atmega" => "ATmega_DFP",
            _ => panic!("{} is not a valid controller family!", pack.name),
        };

        Ok(DownloadablePack::new(
            pack.name.clone(),
            format!(
                "{}Microchip.{pack_name}.{}.atpack",
                DownloadablePacks::PACKS_URL,
                DownloadablePack::VERSION_PLACEHOLDER
            ),
            pack.selected_version.clone(),
            parse_available_versions_from_html(self.html_page.as_str(), pack_name)?,
        ))
    }
}

fn parse_available_versions_from_html(
    html: &str,
    pack_name: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let re = Regex::new(&format!(
        r#"Microchip\.{pack_name}\.(\d*\.\d*\.\d*)\.atpack"#
    ))?;
    Ok(re.captures_iter(html).map(|captures| captures[1].to_string()).collect())
}

#[derive(Debug, PartialEq)]
pub struct DownloadablePack {
    name: String,
    path_template: String,
    selected_version: Option<String>,
    available_versions: Vec<String>,
}

impl DownloadablePack {
    const VERSION_PLACEHOLDER: &str = "{version}";

    pub fn new(
        name: String,
        path_template: String,
        selected_version: Option<String>,
        available_versions: Vec<String>,
    ) -> DownloadablePack {
        println!("{available_versions:?}");
        if selected_version.is_some()
            && !available_versions.contains(selected_version.as_ref().unwrap())
        {
            panic!(
                "pack '{}' does not support selected version {}. Possible versions are: {:?}",
                name,
                selected_version.as_ref().unwrap(),
                available_versions
            )
        }
        DownloadablePack {
            name,
            path_template,
            available_versions,
            selected_version,
        }
    }

    fn url(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.path_template.replace(
            DownloadablePack::VERSION_PLACEHOLDER,
            self.selected_version
                .as_ref()
                .or(self.available_versions.iter().max())
                .unwrap(),
        );
        Ok(url)
    }

    fn download(&self) {
        panic!("Not implemented!")
    }
}

#[cfg(test)]
mod tests {
    use crate::packs::parse_available_versions_from_html;

    #[test]
    fn parse_available_versions_from_html_returns_every_contained_version_for_pack() {
        let versions = parse_available_versions_from_html(
            "BobAndMarry,\nMicrochip.ATmega_DFP.1.atpack//Microchip.ATmega_DFP.1.2.4.atpack",
            "ATmega_DFP",
        )
        .unwrap();
        assert_eq!(versions, vec!("1.2.4"))
    }
}

#[cfg(test)]
mod pack_tests {
    use crate::packs::PackInfo;

    #[test]
    fn parse_controller_detects_name_and_version() {
        assert_eq!(
            PackInfo::from_str("Foo==1.1"),
            PackInfo {
                name: "Foo".to_string(),
                selected_version: Some("1.1".to_string())
            }
        );
    }

    #[test]
    fn from_str_returns_none_version_if_omitted() {
        assert_eq!(
            PackInfo::from_str("Foo"),
            PackInfo {
                name: "Foo".to_string(),
                selected_version: None
            }
        );
    }
}

#[cfg(test)]
mod downloadable_packs_tests {
    use crate::packs::{DownloadablePacks, PackInfo};

    #[test]
    #[should_panic]
    fn for_pack_panics_if_controller_not_supported() {
        DownloadablePacks {
            html_page: "BobAndMarry".to_string(),
        }
        .for_pack(&PackInfo::from_name("hurgel"))
        .unwrap();
    }

    #[test]
    fn for_pack_finds_every_version_for_family() {
        let packs = DownloadablePacks {
            html_page:
                "BobAndMarry,\nMicrochip.ATmega_DFP.1.atpack//Microchip.ATmega_DFP.1.2.4.atpack"
                    .to_string(),
        }
        .for_pack(&PackInfo::from_name("atmega"))
        .unwrap();
        assert_eq!(packs.available_versions, vec!("1.2.4"))
    }
}

#[cfg(test)]
mod downloadable_pack_test {
    use crate::packs::{DownloadablePack, DownloadablePacks, PackInfo};

    #[test]
    #[should_panic]
    fn new_panics_on_unsupported_version_selected() {
        DownloadablePack::new(
            "Bob".to_string(),
            "".to_string(),
            Some("1.1.1".to_string()),
            vec!["".to_string()],
        );
    }

    fn check_url(pack: &DownloadablePack, expected_url: &str) {
        assert_eq!(pack.url().unwrap(), expected_url)
    }

    #[test]
    fn url_returns_download_url_for_selected_version_if_present() {
        let pack = DownloadablePacks {
            html_page: "BobAndMarry,\nMicrochip.ATmega_DFP.1.1.2.atpack\
        //Microchip.ATmega_DFP.1.2.4.atpack"
                .to_string(),
        }
        .for_pack(&PackInfo::from("atmega", "1.1.2"))
        .unwrap();
        check_url(
            &pack,
            "https://packs.download.microchip.com/Microchip.ATmega_DFP.1.1.2.atpack",
        )
    }

    #[test]
    fn url_returns_newest_version_if_none_selected() {
        let pack = DownloadablePacks {
            html_page: "BobAndMarry,\nMicrochip.ATmega_DFP.1.1.2.atpack\
        //Microchip.ATmega_DFP.1.2.4.atpack"
                .to_string(),
        }
        .for_pack(&PackInfo::from_str("atmega"))
        .unwrap();
        check_url(
            &pack,
            "https://packs.download.microchip.com/Microchip.ATmega_DFP.1.2.4.atpack",
        )
    }
}
