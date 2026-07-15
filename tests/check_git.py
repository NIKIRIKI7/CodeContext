import asyncio
import os
from src.services.file_service import FileService, get_git_status_async

async def check_git_logic():
    print("Checking Git status logic...")
    # This assumes we are in a git repo (which we are)
    repo_path = os.getcwd()
    status = await get_git_status_async(repo_path)

    # We don't necessarily have modified files, but we should get a dict
    assert isinstance(status, dict), "Status should be a dictionary"
    print(f"✅ Git status retrieved: {len(status)} files with status.")

    svc = FileService()
    # Mock scan with git
    files = await svc.scan_folders_async(
        paths=[repo_path],
        extensions_str=".py .md",
        ignored_str="node_modules,venv",
        use_git=True,
        use_gitignore=True
    )
    assert isinstance(files, list), "Files should be a list"
    print(f"✅ Scan with Git successful: {len(files)} files found.")

if __name__ == "__main__":
    asyncio.run(check_git_logic())
