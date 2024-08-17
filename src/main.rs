use libloading::{Library, Symbol};
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use zip::ZipArchive;

type CreatePlugin = unsafe extern "C" fn() -> *mut c_void;
type ExecutePlugin = unsafe extern "C" fn(*mut c_void);
type DestroyPluginInstance = unsafe extern "C" fn(*mut c_void);
type MyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum PluginCommand {
    Load(PathBuf),
    Unload(String),
}
#[derive(Deserialize)]
struct ConfigData {
    name: String,
}

struct PluginManager {
    plugins: HashMap<String, (*mut c_void, Library)>,
}

impl PluginManager {
    fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
        }
    }

    fn load_plugin(&mut self, plugin_dir: &Path) -> MyResult<()> {
        let config_path = plugin_dir.join("config.json");
        let config_data = fs::read_to_string(config_path)?;
        let config: ConfigData = serde_json::from_str(&config_data)?;
        let lib_path = plugin_dir.join(get_library_path(&config.name));

        unsafe {
            let lib = Library::new(lib_path)?;
            let create_plugin: Symbol<CreatePlugin> = lib.get(b"create_plugin")?;
            let plugin_instance = create_plugin();

            // Insert the plugin and library into the HashMap first
            self.plugins
                .insert(config.name.clone(), (plugin_instance, lib));

            // Now, retrieve the library and call execute_plugin
            if let Some((plugin_instance, lib)) = self.plugins.get(&config.name) {
                let execute_plugin: Symbol<ExecutePlugin> = lib.get(b"execute_plugin")?;
                execute_plugin(*plugin_instance);
            }
        }

        Ok(())
    }

    fn unload_plugin(&mut self, name: &str) {
        if let Some((plugin_instance, _lib)) = self.plugins.remove(name) {
            unsafe {
                let destroy_plugin_instance: Symbol<DestroyPluginInstance> = _lib
                    .get(b"destroy_plugin_instance")
                    .expect("Failed to load destroy_plugin_instance");
                destroy_plugin_instance(plugin_instance);
            }
            println!("Plugin {} unloaded.", name);

            // Now, remove the plugin's directory
            let plugin_dir = Path::new("dist").join(name);
            if fs::remove_dir_all(plugin_dir).is_err() {
                eprintln!("Failed to remove directory for plugin {}", name);
            } else {
                println!("Directory for plugin {} removed.", name);
            }
        } else {
            // Now, remove the plugin's directory
            let plugin_dir = Path::new("dist").join(name);
            if fs::remove_dir_all(plugin_dir).is_err() {
                eprintln!("Failed to remove directory for plugin {}", name);
            } else {
                println!("Directory for plugin {} removed.", name);
            }
            println!("Plugin {} not found.", name);
        }
    }
    fn load_all_plugins(&mut self, plugins_dir: &Path) -> MyResult<()> {
        for entry in fs::read_dir(plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("plugin") {
                // If a .plugin file is found, extract it first
                let extract_dir = path.with_extension("");
                if let Err(e) = extract_plugin(&path, &extract_dir) {
                    eprintln!("Failed to extract plugin: {}", e);
                    continue;
                }
                // Load the extracted plugin
                if let Err(e) = self.load_plugin(&extract_dir) {
                    eprintln!("Failed to load plugin from {:?}: {}", extract_dir, e);
                }
            } else if path.is_dir() {
                // If a directory is found, attempt to load it directly
                if let Err(e) = self.load_plugin(&path) {
                    eprintln!("Failed to load plugin from {:?}: {}", path, e);
                }
            }
        }
        Ok(())
    }
}

fn extract_plugin(plugin_path: &PathBuf, extract_dir: &PathBuf) -> io::Result<()> {
    println!("Extracting plugin...");
    let file = fs::File::open(plugin_path)?;
    let mut archive = ZipArchive::new(file)?;
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

fn main() -> MyResult<()> {
    let plugins_dir = Path::new("dist");
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);

    // Setup Ctrl+C handler to safely exit the loop
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    let (command_tx, command_rx) = channel::<PluginCommand>();

    // 创建一个 PluginManager 并放入一个新线程中
    std::thread::spawn(move || {
        let mut manager = PluginManager::new();
        // Load all existing plugins in the directory
        if let Err(e) = manager.load_all_plugins(plugins_dir) {
            eprintln!("Failed to load existing plugins: {}", e);
        }
        while let Ok(command) = command_rx.recv() {
            match command {
                PluginCommand::Load(plugin_dir) => {
                    if let Err(e) = manager.load_plugin(&plugin_dir) {
                        eprintln!("Failed to load plugin: {}", e);
                    }
                }
                PluginCommand::Unload(name) => {
                    manager.unload_plugin(&name);
                } // 处理其他命令
            }
        }
    });
    let mut watcher = RecommendedWatcher::new(
        move |res: NotifyResult<Event>| match res {
            Ok(event) => match event.kind {
                EventKind::Modify(_) | EventKind::Remove(_) => {
                    for path in event.paths {
                        if path.extension().and_then(|s| s.to_str()) == Some("plugin") {
                            println!("New plugin detected: {:?}", path);
                            let extract_dir = path.with_extension("");

                            // 先嘗試解壓縮插件，僅在失敗時執行卸載邏輯
                            if let Err(e) = extract_plugin(&path, &extract_dir) {
                                let mut check: bool = false;

                                if let Some(file_name) = extract_dir.file_name() {
                                    if let Err(unload_err) = command_tx.send(PluginCommand::Unload(
                                        file_name.to_str().unwrap().to_string(),
                                    )) {
                                        eprintln!(
                                            "Failed to send unload command to manager thread: {}",
                                            unload_err
                                        );
                                        check = true;
                                    }
                                }
                                if check {
                                    eprintln!("Failed to extract plugin: {}", e);
                                }
                            } else if let Err(e) = command_tx.send(PluginCommand::Load(extract_dir))
                            {
                                eprintln!(
                                    "Failed to send plugin directory to manager thread: {}",
                                    e
                                );
                            }
                        }
                    }
                }

                _ => {}
            },
            Err(e) => eprintln!("watch error: {:?}", e),
        },
        Config::default(),
    )?;

    watcher.watch(plugins_dir, RecursiveMode::NonRecursive)?;

    println!("Watching directory: {:?}", plugins_dir);

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
    }

    println!("Shutting down gracefully...");
    Ok(())
}
