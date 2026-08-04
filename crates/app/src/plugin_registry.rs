use crate::plugin_manager::PluginManager;

pub fn register_all_plugins(pm: &mut PluginManager) {
    // [WIDGET_CLI_INJECT_PLUGINS_START]
    pm.register(sticky_plugin::create_plugin());
    pm.register(todo_plugin::create_plugin());
    pm.register(pet_plugin::create_plugin());
    // [WIDGET_CLI_INJECT_PLUGINS_END]
}
