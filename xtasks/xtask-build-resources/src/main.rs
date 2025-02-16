use anyhow::Context;
use std::env;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use zip::write::SimpleFileOptions;

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) -> anyhow::Result<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path_as_string:?} as {name:?} ...");
            zip.add_directory(path_as_string, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn main() {
    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("root_dir: {root_dir:?}");
    let root_dir = Path::new(&root_dir).join("../../");
    let src_dir = root_dir.join("resources");
    let dst = root_dir.join("resources.zip");

    let dst_file = File::create(dst).unwrap();

    let walkdir = WalkDir::new(&src_dir);
    let it = walkdir.into_iter();

    zip_dir(
        &mut it.filter_map(|e| e.ok()),
        &src_dir,
        dst_file,
        zip::CompressionMethod::Zstd,
    )
    .expect("failed to zip resources folder");
}
