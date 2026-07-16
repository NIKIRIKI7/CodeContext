import asyncio
import sys
import os

# Ensure the root is in the path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

async def check_git_logic():
    print("Checking git integration logic...")
    from src.services.file_service import get_git_status_async, get_git_changed_files_async

    # We don't necessarily need a real git repo for a basic capability check,
    # but we can verify the functions exist and are awaitable.
    repo_path = os.getcwd()

    print(f"Testing get_git_status_async for: {repo_path}")
    status = await get_git_status_async(repo_path)
    assert isinstance(status, dict), "Git status should return a dict"
    print(f"✅ get_git_status_async works (found {len(status)} entries).")

    print(f"Testing get_git_changed_files_async for: {repo_path}")
    files = await get_git_changed_files_async(repo_path, [".py"], set())
    assert isinstance(files, list), "Git changed files should return a list"
    print(f"✅ get_git_changed_files_async works (found {len(files)} files).")

if __name__ == "__main__":
    asyncio.run(check_git_logic())
