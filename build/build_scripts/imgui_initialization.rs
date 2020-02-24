use super::Error;

pub fn initialize_imgui() -> Result<(), Error> {
    std::fs::create_dir_all("dev/imgui")?;
    let our_paths = vec![
        "dev/imgui/imgui_ini.ini",
        "dev/imgui/imgui_log.txt",
        "dev/imgui/meta_data.yaml",
    ];
    let paths: Vec<&std::path::Path> = our_paths.iter().map(|path| std::path::Path::new(path)).collect();
    for this_path in paths {
        if this_path.exists() == false {
            std::fs::File::create(this_path)?;
        }
    }

    Ok(())
}
