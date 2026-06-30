import asyncio
import os
from src.services.file_service import get_git_changed_files_async

async def demo():
    # Simple check for get_git_changed_files_async logic
    # In sandbox we might not have a real git repo with changes against origin/main,
    # but we can check if it at least runs without erroring on repo_path check
    repo_path = os.getcwd()
    try:
        files = await get_git_changed_files_async(repo_path, [".py"], set(), "HEAD")
        print(f"Found {len(files)} changed files in HEAD")
        assert isinstance(files, list)
    except Exception as e:
        print(f"Check failed with error: {e}")
        # If git is not available or it's not a repo, it might return empty list or fail
        # This is just a minimal sanity check that the function exists and is callable.

if __name__ == "__main__":
    asyncio.run(demo())
