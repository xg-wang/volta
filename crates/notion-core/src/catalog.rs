use std::path::Path;
use std::fs::{File, remove_dir_all};
use std::io::{self, Write};
use std::str::FromStr;
use std::string::ToString;

use readext::ReadExt;
use toml;

use path::{self, user_catalog_file};
use provision;
use failure;
use semver::Version;
use serial;

pub struct Catalog {
    pub node: NodeCatalog
}

pub struct NodeCatalog {
    pub current: Option<Version>,
    pub versions: Vec<Version>
}

pub enum Installed {
    Already,
    Now
}

impl Catalog {

    pub fn current() -> Result<Catalog, failure::Error> {
        let path = user_catalog_file()?;
        let src = File::open(path)?.read_into_string()?;
        src.parse()
    }

    pub fn to_string(&self) -> String {
        toml::to_string_pretty(&self.to_serial()).unwrap()
    }

    fn save(&self, path: &Path) -> Result<(), failure::Error> {
        let mut file = File::create(path)?;
        file.write_all(self.to_string().as_bytes())?;
        Ok(())
    }

    pub fn set_version(&mut self, version: Version) -> Result<(), failure::Error> {
        self.install(&version.to_string())?;
        self.node.current = Some(version);
        self.save(&user_catalog_file()?)?;
        Ok(())
    }

    // FIXME: this should take semver::Version instead
    pub fn install(&mut self, version: &str) -> Result<Installed, failure::Error> {
        // FIXME: this should be based on the data structure instead
        if path::node_version_dir(version)?.is_dir() {
            Ok(Installed::Already)
        } else {
            let dest = path::node_versions_dir()?;
            provision::by_version(&dest, version)?;
            // FIXME: update the data structure and self.save()
            Ok(Installed::Now)
        }
    }

    // FIXME: this should take semver::Version instead
    pub fn uninstall(&mut self, version: &str) -> Result<(), failure::Error> {
        let home = path::node_version_dir(version)?;

        // FIXME: this should be based on the data structure instead
        if !home.is_dir() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not a directory", home.to_string_lossy())))?;
        }

        remove_dir_all(home)?;

        // FIXME: update the data structure and self.save()

        Ok(())
    }

}

impl FromStr for Catalog {
    type Err = failure::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let serial: serial::catalog::Catalog = toml::from_str(src)?;
        Ok(serial.into_catalog()?)
    }
}