import pytest
import asyncio
from unittest.mock import patch, AsyncMock, MagicMock, PropertyMock

from src.services.file_service import FileService, get_git_changed_files_async, get_git_status_async
from src.use_cases.scan_use_case import ScanWorkspaceUseCase
from src.store.state import AppState


@pytest.mark.asyncio
async def test_get_git_changed_files_uses_git_base():
    """get_git_changed_files_async подставляет git_base вместо HEAD в git diff."""
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

        mock_exec.assert_any_call(
            "git", "-c", "core.quotepath=false", "diff", "origin/main", "--name-only",
            cwd="/fake/repo", stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE
        )


@pytest.mark.asyncio
async def test_get_git_changed_files_falls_back_to_head():
    """Без git_base использует HEAD."""
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

        mock_exec.assert_any_call(
            "git", "-c", "core.quotepath=false", "diff", "HEAD", "--name-only",
            cwd="/fake/repo", stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE
        )


@pytest.mark.asyncio
async def test_use_case_passes_git_base_to_service():
    """Use Case достаёт git_base из state и передаёт в FileService."""
    state = AppState()
    state.settings.use_git = True
    state.settings.git_base = "origin/develop"
    state.selected_folders = ["/fake/repo"]

    mock_file_service = AsyncMock(spec=FileService)
    mock_file_service.scan_folders_async.return_value = []

    with patch("src.services.file_service.get_git_status_async", return_value={}):
        uc = ScanWorkspaceUseCase(state, mock_file_service)
        await uc.execute(state)

    mock_file_service.scan_folders_async.assert_called_once_with(
        paths=["/fake/repo"],
        extensions_str=state.settings.extensions,
        ignored_str=state.settings.ignored_paths,
        use_git=True,
        use_gitignore=True,
        git_base="origin/develop",
    )


@pytest.mark.asyncio
async def test_file_service_passes_git_base():
    """FileService передаёт git_base при use_git=True."""
    svc = FileService()

    with patch("os.path.isdir", return_value=True), \
         patch("os.path.isfile", return_value=False), \
         patch("src.services.file_service.get_git_changed_files_async", return_value=["/fake/repo/file1.py"]) as mock_git:
        result = await svc.scan_folders_async(
            paths=["/fake/repo"],
            extensions_str=".py",
            ignored_str="",
            use_git=True,
            use_gitignore=False,
            git_base="origin/main",
        )

    import os
    expected_path = os.path.abspath("/fake/repo")
    assert result == ["/fake/repo/file1.py"]
    mock_git.assert_called_once_with(expected_path, [".py"], set(), "origin/main")
