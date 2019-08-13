use log::*;
use std::ffi::OsString;
use std::path::PathBuf;

fn find_it_in(to_find: &str, path: PathBuf) -> Option<PathBuf> {
    debug!("Searching {:?} for {}", path, to_find);
    if path.is_dir() {
        let read_dir = path.read_dir().expect("Could not read directory");
        for sub_path in read_dir {
            let entry = sub_path.expect("Failed to get entry");
            if let Some(f) = find_it_in(to_find, entry.path()) {
                return Some(f);
            }
        }
    } else if path.is_file() {
        if let Some(os_str) = path.file_name() {
            if os_str == to_find {
                return Some(path);
            }
        }
    }
    None
}

fn find_it(path: OsString, to_find: &str) -> Option<PathBuf> {
    std::env::split_paths(&path)
        .filter_map(|dir| find_it_in(to_find, dir))
        .next()
}

fn locate_header_from_package(
    path: OsString,
    header_name: &str,
    package: Package,
) -> Option<PathBuf> {
    debug!(
        "Checking package {} @ {} for {}",
        package.name, package.version, header_name
    );
    if let Ok(library) = pkg_config::Config::new()
        .atleast_version(&package.version)
        .probe(&package.name)
    {
        for include_path in library.include_paths {
            if let Some(f) = find_it_in(header_name, include_path) {
                return Some(f);
            }
        }
    }
    debug!("Could not find package, checking path");
    if let Some(f) = find_it(path, header_name) {
        return Some(f);
    }
    None
}

pub struct Package {
    pub version: String,
    pub name: String,
}

pub fn locate_header(header_name: &str, package: Option<Package>) -> Option<PathBuf> {
    let path = std::env::var_os("PATH").expect("No path defined");
    debug!("Locating header using path {:?}", path);
    locate_header_with_path(path, header_name, package)
}

pub fn locate_header_with_path(
    path: OsString,
    header_name: &str,
    package: Option<Package>,
) -> Option<PathBuf> {
    if let Some(p) = package {
        return locate_header_from_package(path, header_name, p);
    } else {
        return find_it(path, header_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locate_resource_header() {
        let _ = env_logger::try_init();

        let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        assert!(locate_header_with_path(cargo_dir.into_os_string(), "to_find.h", None).is_some());
    }
}
