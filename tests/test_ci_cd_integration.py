import pytest
import asyncio
from unittest.mock import patch, AsyncMock, MagicMock, PropertyMock

from src.data.file_system_repository import FileSystemRepository
from src.services.file_service import FileService
from src.use_cases.scan_use_case import ScanWorkspaceUseCase
from src.store.state import AppState


@pytest.mark.asyncio
async def test_repository_uses_git_base():
    """Репозиторий подставляет git_base вместо HEAD в git diff."""
    repo = FileSystemRepository()

    with patch("pathlib.Path.exists", return_value=True), \
         patch("asyncio.create_subprocess_exec") as mock_exec:

        mock_proc = AsyncMock()
        mock_proc.communicate.return_value = (b"file1.py\nfile2.js", b"")
        mock_exec.return_value = mock_proc

        await repo.get_git_changed_files_async(
            repo_path="/fake/repo",
            extensions=[".py", ".js"],
            ignored_substrings=set(),
            git_base="origin/main"
        )

        mock_exec.assert_any_call(
            "git", "-c", "core.quotepath=false", "diff", "origin/main", "--name-only",
            cwd="/fake/repo", stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE
        )


@pytest.mark.asyncio
async def test_repository_falls_back_to_head():
    """Без git_base репозиторий использует HEAD."""
    repo = FileSystemRepository()

    with patch("pathlib.Path.exists", return_value=True), \
         patch("asyncio.create_subprocess_exec") as mock_exec:

        mock_proc = AsyncMock()
        mock_proc.communicate.return_value = (b"file1.py", b"")
        mock_exec.return_value = mock_proc

        await repo.get_git_changed_files_async(
            repo_path="/fake/repo",
            extensions=[".py"],
            ignored_substrings=set(),
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
    mock_fs_repo = AsyncMock(spec=FileSystemRepository)
    mock_fs_repo.get_git_status_async.return_value = {}
    mock_fs_repo.get_git_changed_files_async.return_value = []

    uc = ScanWorkspaceUseCase(state, mock_file_service, mock_fs_repo)

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
async def test_file_service_passes_git_base_to_repo():
    """FileService передаёт git_base в репозиторий."""
    mock_repo = AsyncMock(spec=FileSystemRepository)
    mock_repo.get_git_changed_files_async.return_value = ["/fake/repo/file1.py"]

    svc = FileService(mock_repo)

    with patch("os.path.isdir", return_value=True), \
         patch("os.path.isfile", return_value=False):
        result = await svc.scan_folders_async(
            paths=["/fake/repo"],
            extensions_str=".py",
            ignored_str="",
            use_git=True,
            use_gitignore=False,
            git_base="origin/main",
        )

    import os
    assert result == ["/fake/repo/file1.py"]
    mock_repo.get_git_changed_files_async.assert_called_once_with(
        os.path.abspath("/fake/repo"), [".py"], set(), "origin/main"
    )
