import sys
import os

# Minimalist import check as per Ponytail principles.
# No frameworks, just assertions.

def test_imports():
    try:
        from src.store.state import AppState
        from src.services.file_service import FileService
        from src.controllers.cli_controller import CliController
        from src.di_container import DIContainer
        from src.services.integration_service import install_context_menu

        # Test basic instantiation
        state = AppState()
        container = DIContainer()
        assert container.state is not None

        print("✅ Core imports and basic DI instantiation successful.")
    except Exception as e:
        print(f"❌ Import check failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    test_imports()
