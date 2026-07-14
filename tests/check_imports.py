import sys
mods = ["src.services.file_service", "src.services.token_service", "src.controllers.main_controller", "src.store.state"]
for m in mods:
    try: __import__(m); print(f"✓ {m}")
    except Exception as e: print(f"✗ {m}: {e}"); sys.exit(1)
