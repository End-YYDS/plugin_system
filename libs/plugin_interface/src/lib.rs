use std::ffi::c_void;

pub trait Plugin {
    fn name(&self) -> &'static str;
    fn execute(&self);
}
pub struct PluginWrapper {
    plugin: Box<dyn Plugin>,
}

impl PluginWrapper {
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        PluginWrapper { plugin }
    }
}
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create_plugin_instance(plugin: Box<dyn Plugin>) -> *mut c_void {
    let wrapper = Box::new(PluginWrapper::new(plugin));
    Box::into_raw(wrapper) as *mut c_void
}

#[no_mangle]
pub extern "C" fn destroy_plugin_instance(wrapper: *mut c_void) {
    if !wrapper.is_null() {
        unsafe {
            let _ = Box::from_raw(wrapper as *mut PluginWrapper);
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_name(wrapper: *mut c_void) -> *const u8 {
    if wrapper.is_null() {
        return std::ptr::null();
    }
    let wrapper = unsafe { &*(wrapper as *mut PluginWrapper) };
    wrapper.plugin.name().as_ptr()
}

#[no_mangle]
pub extern "C" fn execute_plugin(wrapper: *mut c_void) {
    if !wrapper.is_null() {
        let wrapper = unsafe { &*(wrapper as *mut PluginWrapper) };
        wrapper.plugin.execute();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            "TestPlugin"
        }

        fn execute(&self) {
            println!("TestPlugin executed");
        }
    }

    #[test]
    fn test_plugin() {
        let plugin = TestPlugin;
        plugin.execute();
        assert_eq!(plugin.name(), "TestPlugin");
    }

    #[test]
    fn test_create_and_destroy_plugin_instance() {
        let plugin = Box::new(TestPlugin);
        let plugin_ptr = create_plugin_instance(plugin);

        assert!(!plugin_ptr.is_null());

        // Test destroying the plugin instance
        destroy_plugin_instance(plugin_ptr);
    }
}
