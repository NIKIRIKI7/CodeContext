# Testing Guide

## Requirements

```bash
pip install pytest pytest-asyncio pytest-cov
```

## Run all tests

```bash
pytest tests/
```

## Run with coverage report

```bash
pytest tests/ --cov=src --cov-report=term-missing
```

- `--cov=src` — measure coverage for the `src/` package
- `--cov-report=term-missing` — show missing lines in terminal

## Generate HTML coverage report

```bash
pytest tests/ --cov=src --cov-report=html
```

Open `htmlcov/index.html` in a browser for interactive coverage browser.

## Run a specific test file

```bash
pytest tests/test_services.py -v
pytest tests/test_use_cases.py -v
```

## Run a specific test

```bash
pytest tests/test_services.py::test_cleaner_service_py -v
```

## Combine all flags

```bash
pytest tests/ --cov=src --cov-report=term-missing --cov-report=html -v
```

## Test structure

- `tests/conftest.py` — shared fixtures (mocks for Dispatcher, FileSystemRepository, SettingsRepository, AppState)
- `tests/test_services.py` — unit tests for services (cleaner, skeleton, token, patch, formatting, dependency, tour)
- `tests/test_use_cases.py` — unit tests for use cases (scan, process, patch, github, settings)

Services and use cases are tested in isolation with mocked external dependencies (disk I/O, Git, API calls).
