import sys
import os

def check_imports():
    print("Checking core imports...")
    try:
        from src.store.state import AppState
        from src.controllers.main_controller import MainController
        from src.services.file_service import FileService
        from src.use_cases.scan_use_case import scan_workspace
        from src.utils.pipeline_utils import process_files_batch_parallel
        print("✅ Core imports successful.")
    except ImportError as e:
        print(f"❌ Import failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    check_imports()
