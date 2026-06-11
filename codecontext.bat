@echo off
set "SCRIPT_DIR=%~dp0"

if exist "%SCRIPT_DIR%venv\Scripts\activate.bat" (
    call "%SCRIPT_DIR%venv\Scripts\activate.bat"
) else if exist "%SCRIPT_DIR%.venv\Scripts\activate.bat" (
    call "%SCRIPT_DIR%.venv\Scripts\activate.bat"
)

python "%SCRIPT_DIR%codecontext.py" %*
