import os
import sys

# Ensure repo root is in python path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def test_ponytail_debt_ledger():
    print("Checking Ponytail Debt Ledger...")
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    ledger_path = os.path.join(repo_root, "PONYTAIL-DEBT.md")

    assert os.path.exists(ledger_path), f"Ledger file not found at: {ledger_path}"

    with open(ledger_path, 'r', encoding='utf-8') as f:
        content = f.read()

    assert "# Ponytail Debt Ledger" in content, "Expected header '# Ponytail Debt Ledger' missing."

    lines = [line.strip() for line in content.splitlines() if line.strip()]

    # Check that there are file headers and list items
    file_headers = [line for line in lines if line.startswith("## ")]
    assert len(file_headers) > 0, "No file headers found in the ledger."

    list_items = [line for line in lines if line.startswith("- ")]
    assert len(list_items) > 0, "No ledger entries found."

    # Verify exact formatting of entries
    for item in list_items:
        # Expected pattern: - <file>:<line> — <what>. ceiling: <ceiling>. upgrade: <upgrade>.
        assert " — " in item, f"Entry missing long dash ' — ': {item}"
        assert ". ceiling: " in item, f"Entry missing '. ceiling: ': {item}"
        assert ". upgrade: " in item, f"Entry missing '. upgrade: ': {item}"

        # Strip trailing backticks if present for the suffix assertion
        clean_item = item.rstrip('`').strip()
        assert clean_item.endswith(".") or clean_item.endswith("[no-trigger]"), f"Entry does not end with expected ending: {item}"

    # Verify ending statistics
    last_line = lines[-1]
    assert "markers," in last_line and "with no trigger." in last_line, f"Ending statistics line missing or malformed: {last_line}"

    print(f"✅ Ponytail Debt Ledger check passed ({len(list_items)} entries, {len(file_headers)} files).")

if __name__ == "__main__":
    test_ponytail_debt_ledger()
