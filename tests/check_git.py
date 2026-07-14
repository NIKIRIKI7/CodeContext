import asyncio
from unittest.mock import patch, AsyncMock
from src.services.file_service import get_git_changed_files_async

async def test():
    with patch("pathlib.Path.exists", return_value=True), patch("pathlib.Path.is_file", return_value=True), \
         patch("asyncio.create_subprocess_exec") as m:
        m.return_value = pr = AsyncMock()
        pr.communicate.side_effect = [(b"f1.py", b""), (b"f2.py", b""), (b"f1.py", b""), (b"", b"")]

        res = await get_git_changed_files_async("/repo", [".py"], set(), "main")
        assert len(res) == 2 and "main" in m.call_args_list[0][0]

        res = await get_git_changed_files_async("/repo", [".py"], set())
        assert len(res) == 1 and "HEAD" in m.call_args_list[2][0]
        print("✓ Git logic verified")

if __name__ == "__main__": asyncio.run(test())
