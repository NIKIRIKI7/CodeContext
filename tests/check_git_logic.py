import asyncio
import unittest.mock
import sys
from src.services.file_service import get_git_changed_files_async

async def test_git_logic():
    # ponytail: minimal logic check for git command construction
    # verifies that git_base is correctly passed to the git diff command.
    try:
        with unittest.mock.patch("pathlib.Path.exists", return_value=True), \
             unittest.mock.patch("asyncio.create_subprocess_exec") as mock_exec:

            mock_proc = unittest.mock.AsyncMock()
            mock_proc.communicate.return_value = (b"test.py", b"")
            mock_exec.return_value = mock_proc

            # Check git_base usage
            await get_git_changed_files_async("/tmp", [".py"], set(), "origin/main")

            # Verify the call - we care about the "origin/main" argument
            args, kwargs = mock_exec.call_args_list[0]
            assert "origin/main" in args
            assert "diff" in args

            print("✅ Git logic check passed.")
    except Exception as e:
        print(f"❌ Git logic check failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(test_git_logic())
