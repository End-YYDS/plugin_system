use plugin_interface::Plugin;
use std::ffi::c_void;
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &'static str {
        "Example Plugin"
    }
    fn execute(&self) {
        println!("Hello from ExampleModule!");
    }
}
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut c_void {
    // 創建一個 ExamplePlugin 並包裝在 Box 中，然後通過 FFI 安全的方式返回
    let plugin: Box<dyn Plugin> = Box::new(ExamplePlugin);
    plugin_interface::create_plugin_instance(plugin)
}
