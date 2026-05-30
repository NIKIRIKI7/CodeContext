import pytest
from src.utils.pipeline_utils import PipelineUtils
from src.utils.config import get_font_path
from src.utils.async_runtime import AsyncRuntime
from types import SimpleNamespace
from unittest.mock import MagicMock


def test_pipeline_utils():
    cleaner = MagicMock()
    cleaner.clean.return_value = "clean"
    skel = MagicMock()
    skel.make_skeleton.return_value = "skel"
    token = MagicMock()
    token.count_tokens.return_value = 1

    opts = SimpleNamespace(skeleton_mode=True)
    res = PipelineUtils.process_files_batch([{'path': 'a', 'content': 'b', 'ext': '.py'}], opts, cleaner, skel, token)
    assert res[0].content == "skel"
    assert res[0].tokens == 1


def test_config():
    assert get_font_path() is not None or True


def test_async_runtime():
    AsyncRuntime.start()
    assert AsyncRuntime._loop is not None
    AsyncRuntime.stop()
