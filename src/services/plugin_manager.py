import os
import json
import sys
import importlib.util
from typing import List, Dict

from ..api.plugin_api import PluginAPI, IPlugin, UIRegistry
from ..utils.logger import app_logger
from src.i18n import add_plugin_translations


class PluginManager:
    def __init__(self, store, dispatcher, container, ui_registry: UIRegistry):
        self.store = store
        self.dispatcher = dispatcher
        self.container = container
        self.ui_registry = ui_registry

        self.plugins_dir = os.path.join(os.getcwd(), "plugins")
        self.loaded_plugins: Dict[str, IPlugin] = {}
        os.makedirs(self.plugins_dir, exist_ok=True)

    def discover_plugins(self) -> List[dict]:
        manifests = []
        if not os.path.exists(self.plugins_dir):
            return manifests

        for item in os.listdir(self.plugins_dir):
            plugin_path = os.path.join(self.plugins_dir, item)
            if os.path.isdir(plugin_path):
                manifest_path = os.path.join(plugin_path, "manifest.json")
                if os.path.exists(manifest_path):
                    try:
                        with open(manifest_path, 'r', encoding='utf-8') as f:
                            manifest = json.load(f)
                            manifest['_dir'] = plugin_path
                            manifests.append(manifest)
                    except Exception as e:
                        app_logger.error(f"Error reading manifest in {plugin_path}: {e}")
        return manifests

    def load_plugin(self, manifest: dict):
        p_id = manifest.get("id")
        if not p_id or p_id in self.loaded_plugins:
            return

        entry_point = manifest.get("entry_point")
        if not entry_point:
            app_logger.error(f"Plugin {p_id} has no entry_point")
            return

        module_name, class_name = entry_point.split('.')
        plugin_dir = manifest['_dir']
        module_path = os.path.join(plugin_dir, f"{module_name}.py")

        if not os.path.exists(module_path):
            app_logger.error(f"Plugin module not found: {module_path}")
            return

        try:
            locales_dir = os.path.join(plugin_dir, "locales")
            if os.path.exists(locales_dir):
                for filename in os.listdir(locales_dir):
                    if filename.endswith(".json"):
                        lang_code = filename.replace(".json", "")
                        try:
                            with open(os.path.join(locales_dir, filename), "r", encoding="utf-8-sig") as f:
                                data = json.load(f)
                                add_plugin_translations(lang_code, data)
                        except Exception as e:
                            app_logger.error(f"Error loading translation {filename} for plugin {p_id}: {e}")

            if plugin_dir not in sys.path:
                sys.path.insert(0, plugin_dir)

            spec = importlib.util.spec_from_file_location(f"plugin_{p_id}", module_path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)

            plugin_class = getattr(module, class_name)
            plugin_instance = plugin_class()

            plugin_instance.id = p_id
            plugin_instance.name = manifest.get("name", p_id)
            plugin_instance.version = manifest.get("version", "1.0.0")

            plugin_api = PluginAPI(p_id, self.store, self.dispatcher, self.container, self.ui_registry)
            plugin_instance.on_init(plugin_api)

            self.loaded_plugins[p_id] = plugin_instance
            app_logger.info(f"Loaded plugin: {plugin_instance.name} v{plugin_instance.version}")
        except Exception as e:
            app_logger.error(f"Failed to load plugin {p_id}: {e}")

    def shutdown_all(self):
        for p_id, plugin in self.loaded_plugins.items():
            try:
                plugin.on_shutdown()
            except Exception as e:
                app_logger.error(f"Error shutting down plugin {p_id}: {e}")
