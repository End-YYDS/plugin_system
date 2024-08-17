use libloading::{Library, Symbol};
use serde::Deserialize;
use std::ffi::c_void;
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};
use zip::ZipArchive;
type CreatePlugin = unsafe extern "C" fn() -> *mut c_void;
#[derive(Deserialize)]
struct Config {
    name: String,
}

fn extract_plugin(plugin_path: &PathBuf, extract_dir: &PathBuf) -> io::Result<()> {
    println!("Extracting plugin...");
    println!("plugin_path: {:?}", plugin_path);

    println!("extract_dir: {:?}", extract_dir);
    let file = fs::File::open(plugin_path)?;
    let mut archive = ZipArchive::new(file)?;
    let path = plugin_path.file_name().unwrap();
    println!("Extracting plugin to {:?}", path);
    archive.extract(extract_dir)?;
    Ok(())
}

fn get_library_path(plugin_name: &str) -> PathBuf {
    let lib_file = format!(
        "lib{}.{}",
        plugin_name,
        if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        }
    );
    PathBuf::from(lib_file)
}

fn load_and_execute_plugin(plugin_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = plugin_dir.join("config.json");
    let config_data = fs::read_to_string(config_path)?;
    let config: Config = serde_json::from_str(&config_data)?;
    let lib_path = Path::new(plugin_dir).join(get_library_path(&config.name));
    let lib_entry = b"create_plugin";
    unsafe {
        let lib = Library::new(lib_path)?;
        let func: Symbol<CreatePlugin> = lib.get(lib_entry)?;
        let plugin_instance = func();

        // 調用插件的 execute_plugin 方法
        plugin_interface::execute_plugin(plugin_instance);

        // 銷毀插件實例
        plugin_interface::destroy_plugin_instance(plugin_instance);
    }

    Ok(())
}

fn main() {
    let plugins_dir = Path::new("dist"); // 插件目錄
    let plugin_path = plugins_dir.join("example.plugin"); // 插件壓縮包的路徑
    let extract_dir = plugin_path.with_extension("");

    // 提取插件壓縮包
    if let Err(e) = extract_plugin(&plugin_path, &extract_dir) {
        eprintln!("Failed to extract plugin: {}", e);
        return;
    }

    // 加載並執行插件
    if let Err(e) = load_and_execute_plugin(&extract_dir) {
        eprintln!("Failed to load and execute plugin: {}", e);
    }
}
