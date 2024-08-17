# 如何創建一個新的插件
:::info
每個插件都需要繼承Plugin這個trait裡
:::
在plugins內執行```cargo new --lib {插件名}```
## 創建後
進入`{插件名}/src/lib.rs`開始編輯
```rust
use plugin_interface::Plugin;
use std::ffi::c_void;
pub struct ExamplePlugin; // 插件名

impl Plugin for ExamplePlugin {
    fn name(&self) -> &'static str {
        "Example Plugin"
    }
    fn execute(&self) {
        println!("Hello from ExampleModule!");
    }
}
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut c_void { // 主要進入點
    // 創建一個 ExamplePlugin 並包裝在 Box 中，然後通過 FFI 安全的方式返回
    let plugin: Box<dyn Plugin> = Box::new(ExamplePlugin);
    plugin_interface::create_plugin_instance(plugin)
}
```
## 在新建的crate下新增一個config.json
內容如下
```json
{
    "name": "example_plugin"
}
```
## 在Cargo.toml中添加
```toml
[lib]
crate-type = ["cdylib"]
[dependencies]
plugin_interface = { workspace = true }
```


## 提供的API
```rust
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn execute(&self);
}
```

## 目錄詳解
- `plugins`是插件開發的目錄
- `dist`是最後編譯出來的插件，副檔名為.plugin，程序會自動讀入

## 如何編譯
執行```make```就可以編譯及執行