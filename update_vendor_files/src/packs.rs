use std::io::Write;

use regex::Regex;
use vfs::{MemoryFS, VfsPath};
use zip::ZipArchive;

// TODO improve error handling: at least swap panics with result errors?

#[derive(PartialEq, Debug)]
pub struct PackInfo {
    pub name: String,
    name_for_download: String,
    pub selected_version: Option<String>,
}

impl PackInfo {
    pub fn from_str(arg: &str) -> PackInfo {
        match arg.split_once("==") {
            Some((name, version)) => Self::from(name, version),
            None => Self::from_name(arg),
        }
    }

    fn from(name: &str, version: &str) -> PackInfo {
        PackInfo {
            name: name.to_string(),
            name_for_download: Self::name_for_download(name).unwrap().to_string(),
            selected_version: Some(version.to_string()),
        }
    }

    fn from_name(name: &str) -> PackInfo {
        PackInfo {
            name: name.to_string(),
            name_for_download: Self::name_for_download(name).unwrap().to_string(),
            selected_version: None,
        }
    }

    fn name_for_download(name: &str) -> Option<&str> {
        // The whole project uses the lowercase representation, so we do that too and convert it here
        // Additionally, those packs have some suffixes which are applied here
        match name {
            "atmega" => Some("Microchip.ATmega_DFP"),
            _ => None,
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

    pub fn for_pack(&self, pack: PackInfo) -> Result<DownloadablePack, Box<dyn std::error::Error>> {
        let available_versions =
            parse_available_versions_from_html(self.html_page.as_str(), &pack.name_for_download)?;
        Ok(DownloadablePack::new(pack, available_versions))
    }
}

fn parse_available_versions_from_html(
    html: &str,
    pack_name: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let prefix = pack_name.replace('.', r"\."); // dots have to be escaped in a regex
    let suffix = DownloadablePack::ATPACK_SUFFIX;
    let re = Regex::new(&format!(r"{prefix}\.(\d*\.\d*\.\d*)\.{suffix}"))?;
    Ok(re
        .captures_iter(html)
        .map(|captures| captures[1].to_string())
        .collect())
}

#[derive(Debug, PartialEq)]
pub struct DownloadablePack {
    pack_info: PackInfo,
    available_versions: Vec<String>,
}

impl DownloadablePack {
    const ATPACK_SUFFIX: &str = "atpack";
    pub fn new(pack_info: PackInfo, available_versions: Vec<String>) -> DownloadablePack {
        let name = &pack_info.name;
        let selected_version = &pack_info.selected_version;
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
            pack_info,
            available_versions,
        }
    }

    fn url(&self) -> String {
        let version = self.version();
        let prefix = DownloadablePacks::PACKS_URL.to_owned() + &self.pack_info.name_for_download;
        let suffix = Self::ATPACK_SUFFIX;
        format!("{prefix}.{version}.{suffix}")
    }

    fn version(&self) -> &str {
        self.pack_info
            .selected_version
            .as_ref()
            .or(self.available_versions.iter().max())
            .unwrap()
    }

    pub fn download(&self) -> Result<Pack, Box<dyn std::error::Error>> {
        let url = self.url();
        println!("Downloading {}", url);
        let zip: VfsPath = MemoryFS::new().into();
        zip.join("packs.zip")?
            .create_file()?
            .write_all(reqwest::blocking::get(url)?.bytes()?.as_ref())?;
        println!("Downloaded.");
        Ok(Pack { content: zip })
    }
}

pub struct Pack {
    pub content: VfsPath,
}

impl Pack {
    const ATDF_PATH_IN_ZIP: &str = "atdf/";

    pub fn list_controllers(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let zip_file = ZipArchive::new(self.content.join("packs.zip")?.open_file()?)?;

        let mut controllers: Vec<String> = zip_file
            .file_names()
            .filter(|s| s.contains(Pack::ATDF_PATH_IN_ZIP))
            .flat_map(|s| {
                s.strip_prefix("atdf/")
                    .and_then(|s| s.strip_suffix(".atdf"))
            })
            .map(|s| s.to_string())
            .collect();
        controllers.sort();
        Ok(controllers)
    }
}

#[cfg(test)]
mod pack_tests {
    use vfs::{MemoryFS, VfsPath};
    use zip::ZipWriter;

    use crate::packs::Pack;

    fn put_files_in_content(files: &[&str]) -> Result<VfsPath, Box<dyn std::error::Error>> {
        let mut zip_file = ZipWriter::new(std::io::Cursor::new(Vec::<u8>::new()));
        for file in files {
            zip_file.start_file(*file, zip::write::FileOptions::default())?;
        }
        let zip_file = zip_file.finish()?.into_inner();

        let path: VfsPath = MemoryFS::new().into();
        path.join("packs.zip")?.create_file()?.write(&zip_file)?;
        Ok(path)
    }

    fn check_list_controllers(
        files: &[&str],
        expected_controllers: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = put_files_in_content(files)?;
        let pack = Pack { content };
        assert_eq!(pack.list_controllers()?, expected_controllers);
        Ok(())
    }

    macro_rules! generate_list_controller_test {
        ($test_name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
                check_list_controllers($input, $expected)
            }
        };
    }

    generate_list_controller_test!(
        list_controllers_returns_nothing_if_only_ignored_entries_present,
        &["Bob", "Nested/Marry"],
        &[]
    );

    #[test]
    fn list_controllers_returns_atdf_controllers_if_only_atdf_files_present(
    ) -> Result<(), Box<dyn std::error::Error>> {
        check_list_controllers(
            &["atdf/ATmega.atdf", "atdf/ATmega8.atdf"],
            &["ATmega", "ATmega8"],
        )
    }

    #[test]
    fn list_controllers_returns_only_atdf_files_if_many_files_present(
    ) -> Result<(), Box<dyn std::error::Error>> {
        check_list_controllers(
            &[
                "atdf/ATmega.atdf",
                "atdf/ATmega8.atdf",
                "foo/atdf/bargel.foo",
            ],
            &["ATmega", "ATmega8"],
        )
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
mod pack_info_tests {
    use crate::packs::PackInfo;

    #[test]
    fn from_str_detects_name_and_version() {
        let info = PackInfo::from_str("atmega==1.1");
        assert_eq!(info.selected_version, Some("1.1".to_string()));
        assert_eq!(info.name, "atmega");
    }

    #[test]
    fn from_str_returns_none_version_if_omitted() {
        assert_eq!(PackInfo::from_str("atmega").selected_version, None);
    }

    #[test]
    fn name_for_download_returns_some_for_supported() {
        assert_eq!(
            PackInfo::name_for_download("atmega"),
            Some("Microchip.ATmega_DFP")
        );
    }

    #[test]
    fn name_for_download_returns_none_for_unsupported() {
        assert_eq!(
            PackInfo::name_for_download("NoOneNamesAControllerBob_right?"),
            None
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
        .for_pack(PackInfo {
            name: "hurgel".to_string(),
            name_for_download: "hurgel".to_string(),
            selected_version: None,
        })
        .unwrap();
    }

    #[test]
    fn for_pack_finds_every_version_for_family() {
        let packs = DownloadablePacks {
            html_page:
                "BobAndMarry,\nMicrochip.ATmega_DFP.1.atpack//Microchip.ATmega_DFP.1.2.4.atpack"
                    .to_string(),
        }
        .for_pack(PackInfo::from_name("atmega"))
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
        DownloadablePack::new(PackInfo::from_str("atmega==1.1.1"), vec!["".to_string()]);
    }

    fn check_url(pack: &DownloadablePack, expected_url: &str) {
        assert_eq!(pack.url(), expected_url)
    }

    #[test]
    fn url_returns_download_url_for_selected_version_if_present() {
        let pack = DownloadablePacks {
            html_page: "BobAndMarry,\nMicrochip.ATmega_DFP.1.1.2.atpack\
        //Microchip.ATmega_DFP.1.2.4.atpack"
                .to_string(),
        }
        .for_pack(PackInfo::from("atmega", "1.1.2"))
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
        .for_pack(PackInfo::from_str("atmega"))
        .unwrap();
        check_url(
            &pack,
            "https://packs.download.microchip.com/Microchip.ATmega_DFP.1.2.4.atpack",
        )
    }
}
