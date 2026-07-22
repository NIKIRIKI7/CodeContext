import sys
import os

# Ensure the root is in the path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def check_imports():
    print("Checking core imports...")
    try:
        from src.di_container import DIContainer
        from src.store.state import AppState
        from src.controllers.main_controller import MainController
        from src.services.file_service import FileService
        from src.services.formatting_service import format_output
        print("✅ Core imports OK.")
    except ImportError as e:
        print(f"❌ Import failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    check_imports()
