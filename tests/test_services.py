import pytest
import os
from unittest.mock import patch, MagicMock, AsyncMock

from src.services.cleaner_service import CleanerService
from src.services.skeleton_service import SkeletonService
from src.services.token_service import TokenService
from src.services.patch_service import PatchService
from src.services.formatting_service import FormattingService
from src.services.dependency_service import DependencyService
from src.services.tour_service import TourService
from src.store.state import ProcessedFile


# --- CLEANER SERVICE ---

def test_cleaner_service_js(dummy_options):
    service = CleanerService()
    code = "/* block comment */\nconst a = 1; // line comment\n\nconst b = 2;"
    cleaned = service.clean(code, ".js", dummy_options)
    assert "block comment" not in cleaned
    assert "line comment" not in cleaned
    assert cleaned == "const a = 1;\nconst b = 2;"


def test_cleaner_service_py(dummy_options):
    service = CleanerService()
    code = "def foo():\n    # python comment\n    pass\n"
    cleaned = service.clean(code, ".py", dummy_options)
    assert "python comment" not in cleaned
    assert cleaned == "def foo():\npass"


def test_cleaner_service_secrets(dummy_options):
    service = CleanerService()
    code = 'api_key = "AKIA1234567890123456"\nlet auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.secret"'
    cleaned = service.clean(code, ".py", dummy_options)
    assert "AKIA" not in cleaned
    assert "[REDACTED]" in cleaned


# --- SKELETON SERVICE ---

def test_skeleton_service_python():
    service = SkeletonService()
    code = "def hello():\n    '''docstring'''\n    print('world')\n    return True"
    skel = service.make_skeleton(code, ".py")
    assert "print" not in skel
    assert "..." in skel or "Ellipsis" in skel
    assert "docstring" in skel


def test_skeleton_service_python_syntax_error():
    service = SkeletonService()
    code = "def hello(:: # Syntax Error"
    skel = service.make_skeleton(code, ".py")
    assert "Syntax Error:" in skel


def test_skeleton_service_web():
    service = SkeletonService()
    code = "function calculate() {\n  let a = 1;\n  return a;\n}"
    skel = service.make_skeleton(code, ".js")
    assert "let a = 1;" not in skel
    assert "function calculate() { // ... }" in skel


# --- TOKEN SERVICE ---

def test_token_service():
    service = TokenService()
    tokens = service.count_tokens("hello world")
    assert tokens > 0
    service.encoding = None
    assert service.count_tokens("1234") == 1


# --- PATCH SERVICE ---

def test_patch_service_prepare_and_apply(tmp_path):
    service = PatchService()

    dummy_file = tmp_path / "test.py"
    dummy_file.write_text("def old():\n    pass\n", encoding="utf-8")

    patches = [
        {"action": "replace", "file": "test.py", "search": "def old():\n    pass", "content": "def new():\n    return True"},
        {"action": "create", "file": "new.py", "content": "print('hello')"},
        {"action": "delete", "file": "test.py"}
    ]

    prepared = service.prepare_patches(patches, [str(tmp_path)])

    assert len(prepared) == 3
    assert prepared[0]['success'] is True
    assert prepared[0]['action'] == 'replace'
    assert "def new():" in prepared[0]['patched_content']

    assert prepared[1]['success'] is True
    assert prepared[1]['action'] == 'create'

    assert prepared[2]['success'] is True
    assert prepared[2]['action'] == 'delete'

    applied, logs = service.apply_prepared([prepared[1]])
    assert applied == 1
    assert (tmp_path / "new.py").exists()


# --- FORMATTING SERVICE ---

def test_formatting_service():
    service = FormattingService()
    files = [ProcessedFile(path="src/main.py", content="print('hi')", tokens=2)]
    deps = {"src/main.py": {"os", "sys"}}

    markdown = service.format_output(files, "markdown", True, "System Prompt", deps)
    assert "> **System Context:**\n> System Prompt" in markdown
    assert "└── main.py" in markdown
    assert "`os`" in markdown
    assert "```py\nprint('hi')\n```" in markdown

    xml = service.format_output(files, "xml", False, "", None)
    assert '<file path="src/main.py">' in xml
    assert "<content>\nprint(&#x27;hi&#x27;)\n    </content>" in xml


# --- DEPENDENCY SERVICE ---

@pytest.mark.asyncio
async def test_dependency_service():
    service = DependencyService()
    files = [
        {"path": "main.py", "content": "import os\nfrom sys import path", "ext": ".py"},
        {"path": "app.js", "content": "import { ref } from 'vue';\nrequire('axios');", "ext": ".js"}
    ]

    deps = await service.resolve_dependencies(files)

    assert "main.py" in deps
    assert "os" in deps["main.py"]
    assert "sys" in deps["main.py"]

    assert "app.js" in deps
    assert "vue" in deps["app.js"]
    assert "axios" in deps["app.js"]


# --- TOUR SERVICE ---

def test_tour_service():
    service = TourService()
    steps = service.get_tour_steps()
    assert len(steps) > 0
    assert "title" in steps[0]
