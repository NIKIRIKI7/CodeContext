import pytest
from unittest.mock import AsyncMock, patch, MagicMock
from src.services.import_resolution_service import ImportResolutionService
from src.services.integration_service import IntegrationService
from src.services.llm_checker_service import LlmCheckerService
from src.services.processing_service import ProcessingService
from src.services.github_service import GitHubService
from src.services.output_service import OutputService
from src.services.file_service import FileService
from src.services.strategies.integration_strategies import ContextMenuStrategy
from src.store.state import AppSettings


def test_import_resolution():
    srv = ImportResolutionService()
    paths = ["src/utils.py", "src/domain/user/__init__.py", "src/components/Button/Button.tsx", "src/entities/user/index.ts"]
    assert "src/utils.py" in srv.resolve("./utils", paths)
    assert "src/domain/user/__init__.py" in srv.resolve("domain.user", paths)
    assert "src/components/Button/Button.tsx" in srv.resolve("components/Button", paths)
    assert "src/entities/user/index.ts" in srv.resolve("@/entities/user", paths)


class DummyMenu(ContextMenuStrategy):
    def install(self, p=None): return True, "ok"
    def remove(self): return True, "ok"


def test_integration_service():
    srv = IntegrationService(DummyMenu())
    assert srv.install_context_menu()[0] is True
    assert srv.remove_context_menu()[0] is True


@pytest.mark.asyncio
async def test_llm_checker():
    srv = LlmCheckerService()
    st = AppSettings(llm_check_enabled=False)
    res = await srv.check_patch("a", "b", st)
    assert res['status'] == 'DISABLED'

    st.llm_check_enabled = True
    st.llm_base_url = ""
    res = await srv.check_patch("a", "b", st)
    assert res['status'] == 'ERROR'

    st.llm_base_url = "http://test"
    with patch("urllib.request.urlopen") as mock_open:
        m = MagicMock()
        m.read.return_value = b'{"choices":[{"message":{"content":"{\\"status\\":\\"SAFE\\"}"}}]}'
        mock_open.return_value.__enter__.return_value = m
        res = await srv.check_patch("a", "b", st)
        assert res['status'] == 'SAFE'


@pytest.mark.asyncio
async def test_processing_service(tmp_path):
    repo = MagicMock()
    repo.read_file_async = AsyncMock(return_value="code")
    srv = ProcessingService(repo)

    f = tmp_path / "t.py"
    f.write_text("a")
    res = await srv.read_files_async([str(f)])
    assert len(res) == 1
    assert res[0]['content'] == "code"


@pytest.mark.asyncio
async def test_github_service(tmp_path):
    with patch("asyncio.create_subprocess_exec") as mock_exec:
        proc = AsyncMock()
        proc.communicate.return_value = (b"", b"")
        proc.returncode = 0
        mock_exec.return_value = proc

        res = await GitHubService.clone_repo_async("https://test", str(tmp_path / "repo"))
        assert res == str(tmp_path / "repo")


def test_output_service(tmp_path):
    srv = OutputService()
    f = tmp_path / "o.txt"
    srv.save_to_file("txt", str(f))
    assert f.exists()

    pdf = tmp_path / "o.pdf"
    srv.save_to_pdf("pdf", str(pdf))
    assert pdf.exists()

    with patch("pyperclip.copy") as cp:
        srv.copy_to_clipboard("clip")
        cp.assert_called_with("clip")


@pytest.mark.asyncio
async def test_file_service(tmp_path):
    repo = MagicMock()
    repo.walk_directory_async = AsyncMock(return_value=[str(tmp_path / "a.py")])
    repo.read_gitignore.return_value = []
    srv = FileService(repo)

    (tmp_path / "a.py").write_text("1")
    paths = await srv.scan_folders_async([str(tmp_path)], ".py", "node_modules", False, True)
    assert len(paths) >= 1

    (tmp_path / ".git").mkdir()
    root = srv.find_project_root(str(tmp_path / "a.py"))
    assert root == str(tmp_path)
