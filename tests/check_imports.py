import sys

def check_imports():
    core_deps = [
        'PySide6',
        'tiktoken',
        'pyperclip',
        'pathspec',
        'jinja2'
    ]
    failed = False
    for dep in core_deps:
        try:
            __import__(dep)
            print(f"OK: {dep} is importable")
        except ImportError:
            print(f"FAIL: {dep} is NOT importable")
            failed = True

    if failed:
        sys.exit(1)
    else:
        print("All core dependencies are available.")

if __name__ == "__main__":
    check_imports()
