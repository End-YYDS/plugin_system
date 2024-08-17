use plugin_interface::Plugin;
use std::ffi::c_void;
pub struct HelloWorld;

impl Plugin for HelloWorld {
    fn name(&self) -> &'static str {
        "HelloWorld"
    }
    fn execute(&self) {
        println!("Hello World");
    }
}
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut c_void {
    // 創建一個 HelloWorld 並包裝在 Box 中，然後通過 FFI 安全的方式返回
    let plugin: Box<dyn Plugin> = Box::new(HelloWorld);
    plugin_interface::create_plugin_instance(plugin)
}
