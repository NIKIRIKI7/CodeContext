"""Service for Git integration to find modified files."""
import subprocess
from pathlib import Path
from typing import List, Set

class GitService:
    @staticmethod
    def get_changed_files(repo_path: str, extensions: List[str], ignored: Set[str]) -> List[Path]:
        """
        Returns list of modified (unstaged + staged) and untracked files using git.
        """
        files = set()
        repo = Path(repo_path)
        
        if not (repo / ".git").exists():
            return []

        try:
            # 1. Modified & Staged
            cmd_diff = ["git", "diff", "HEAD", "--name-only"]
            output_diff = subprocess.check_output(cmd_diff, cwd=repo_path, text=True)
            
            # 2. Untracked
            cmd_untracked = ["git", "ls-files", "--others", "--exclude-standard"]
            output_untracked = subprocess.check_output(cmd_untracked, cwd=repo_path, text=True)
            
            all_raw = output_diff.splitlines() + output_untracked.splitlines()
            
            for f in all_raw:
                p = repo / f
                if not p.exists() or p.is_dir():
                    continue
                
                # Filter extensions
                if not any(f.endswith(ext) for ext in extensions):
                    continue
                
                # Basic ignore check (partial)
                if any(ign in str(p) for ign in ignored):
                    continue
                    
                files.add(p)
                
        except subprocess.CalledProcessError:
            pass # Not a git repo or error
            
        return list(files)