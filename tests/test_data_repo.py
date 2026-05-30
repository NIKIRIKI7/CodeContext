import os
import pytest
from unittest.mock import patch, AsyncMock
from src.data.file_system_repository import FileSystemRepository
from src.data.settings_repository import SettingsRepository


@pytest.mark.asyncio
async def test_fs_repo(tmp_path):
    repo = FileSystemRepository()

    f = tmp_path / "file.py"
    f.write_text("print(1)")
    assert await repo.read_file_async(str(f)) == "print(1)"
    assert await repo.read_file_async(str(tmp_path / "nope")) is None

    bin_f = tmp_path / "bin.dat"
    bin_f.write_bytes(b"\x00\x01\x02")
    assert await repo.read_file_async(str(bin_f)) is None

    gi = tmp_path / ".gitignore"
    gi.write_text("*.log\nnode_modules")
    ignores = repo.read_gitignore(str(tmp_path))
    assert "*.log\n" in ignores
    assert repo.read_gitignore(str(tmp_path / "nope")) == []

    d = tmp_path / "dir"
    d.mkdir()
    (d / "t.py").write_text("1")
    res = await repo.walk_directory_async(str(tmp_path), set(), [".py"])
    assert len(res) >= 1

    await repo.delete_directory_async(str(d))
    assert not d.exists()

    with patch("asyncio.create_subprocess_exec") as mock_exec:
        proc = AsyncMock()
        proc.communicate.return_value = (b"t.py\n", b"")
        proc.returncode = 0
        mock_exec.return_value = proc

        (tmp_path / ".git").mkdir()
        (tmp_path / "t.py").write_text("1")

        files = await repo.get_git_changed_files_async(str(tmp_path), [".py"], set())
        assert len(files) == 1

        proc.communicate.return_value = (b" M t.py\n", b"")
        status = await repo.get_git_status_async(str(tmp_path))
        assert str(tmp_path / "t.py") in status


def test_settings_repo(tmp_path, monkeypatch):
    monkeypatch.setattr("os.path.dirname", lambda x: str(tmp_path))
    repo = SettingsRepository("test_settings.json")

    assert repo.load() == {}
    repo.save({"minify": True})
    assert repo.load()["minify"] is True
