import asyncio
from unittest.mock import patch, AsyncMock
from src.services.file_service import get_git_changed_files_async

async def check_git_base_usage():
    """Verify that git_base is correctly used in git diff command."""
    with patch("pathlib.Path.exists", return_value=True), \
         patch("asyncio.create_subprocess_exec") as mock_exec:

        mock_proc = AsyncMock()
        mock_proc.communicate.return_value = (b"file1.py\nfile2.js", b"")
        mock_exec.return_value = mock_proc

        await get_git_changed_files_async(
            repo_path="/fake/repo",
            extensions=[".py", ".js"],
            ignored=set(),
            git_base="origin/main"
        )

        # Check if git diff was called with origin/main
        args, kwargs = mock_exec.call_args_list[0]
        assert "origin/main" in args, f"Expected origin/main in args, got {args}"
        assert "--name-only" in args
        print("✓ git_base usage verified.")

async def check_git_default_head():
    """Verify that HEAD is used when git_base is not provided."""
    with patch("pathlib.Path.exists", return_value=True), \
         patch("asyncio.create_subprocess_exec") as mock_exec:

        mock_proc = AsyncMock()
        mock_proc.communicate.return_value = (b"file1.py", b"")
        mock_exec.return_value = mock_proc

        await get_git_changed_files_async(
            repo_path="/fake/repo",
            extensions=[".py"],
            ignored=set(),
        )

        args, kwargs = mock_exec.call_args_list[0]
        assert "HEAD" in args, f"Expected HEAD in args, got {args}"
        print("✓ HEAD default verified.")

if __name__ == "__main__":
    asyncio.run(check_git_base_usage())
    asyncio.run(check_git_default_head())
    print("All Git integration checks passed.")
