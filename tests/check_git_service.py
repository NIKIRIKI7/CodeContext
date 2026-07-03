import asyncio
import os
import shutil
import tempfile
from src.services.file_service import get_git_changed_files_async

async def test_git_logic():
    # ponytail: minimal check for non-trivial git diff logic
    tmp_dir = tempfile.mkdtemp()
    try:
        os.chdir(tmp_dir)
        os.system("git init -q")
        os.system("git config user.email 'you@example.com'")
        os.system("git config user.name 'Your Name'")

        f1 = os.path.join(tmp_dir, "test.py")
        with open(f1, "w") as f: f.write("print(1)")
        os.system("git add test.py && git commit -m 'init' -q")

        with open(f1, "a") as f: f.write("\nprint(2)")

        changed = await get_git_changed_files_async(tmp_dir, [".py"], set())
        assert any(f.endswith("test.py") for f in changed), "Git changed files not detected"
        print("✅ Git changed files check passed")

    finally:
        shutil.rmtree(tmp_dir)

if __name__ == "__main__":
    asyncio.run(test_git_logic())
