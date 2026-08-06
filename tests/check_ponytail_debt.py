import os
import re

def parse_codebase_for_ponytail():
    exclude_dirs = {'.git', 'node_modules', 'build', 'dist', '__pycache__', '.pytest_cache'}
    hits = []

    for root, dirs, files in os.walk('.'):
        dirs[:] = [d for d in dirs if d not in exclude_dirs]
        for file in files:
            if file == 'PONYTAIL-DEBT.md':
                continue
            filepath = os.path.join(root, file)
            if filepath.startswith('./'):
                filepath = filepath[2:]

            try:
                with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                    for i, line in enumerate(f, 1):
                        match = re.search(r'(?:#|//)\s*ponytail:\s*(.*)', line)
                        if match:
                            text = match.group(1).strip()
                            hits.append({
                                'file': filepath,
                                'line': i,
                                'text': text
                            })
            except Exception:
                pass

    hits.sort(key=lambda x: (x['file'], x['line']))
    return hits

def main():
    print("Running assert-based self-check for ponytail debt ledger...")

    # 1. Scan the codebase
    expected_hits = parse_codebase_for_ponytail()
    expected_total = len(expected_hits)
    expected_no_trigger = sum(1 for h in expected_hits if ',' not in h['text'])

    # 2. Read the generated ledger
    ledger_path = 'PONYTAIL-DEBT.md'
    assert os.path.exists(ledger_path), f"Ledger file {ledger_path} does not exist!"

    with open(ledger_path, 'r', encoding='utf-8') as f:
        content = f.read()

    lines = content.splitlines()

    # 3. Assert title exists
    assert lines[0] == "# Ponytail Technical Debt Ledger", "Title of ledger is incorrect or missing!"

    # Filter lines that actually contain file references
    # Format expected: - `file:line` — ...
    entry_lines = [l for l in lines if l.startswith('- `')]

    assert len(entry_lines) == expected_total, f"Expected {expected_total} entries in ledger, found {len(entry_lines)}."

    # 4. Check each entry in detail
    for hit, line in zip(expected_hits, entry_lines):
        ref = f"`{hit['file']}:{hit['line']}`"
        assert ref in line, f"Expected reference {ref} not found in line: {line}"

        # Check trigger tagging
        if ',' in hit['text']:
            assert 'ceiling:' in line and 'upgrade:' in line, f"Expected trigger parsing with ceiling and upgrade for {ref}, but line was: {line}"
            assert '[no-trigger]' not in line, f"Entry {ref} has a trigger but was tagged [no-trigger]: {line}"
        else:
            assert '[no-trigger]' in line, f"Expected [no-trigger] tag for {ref}, but line was: {line}"
            assert 'ceiling:' not in line, f"Entry {ref} should not have a ceiling tag: {line}"

    # 5. Check footer totals
    footer_pattern = re.compile(r'(\d+)\s+markers,\s+(\d+)\s+with no trigger\.')
    footer_match = None
    for line in reversed(lines):
        if line.strip():
            footer_match = footer_pattern.search(line)
            if footer_match:
                break

    assert footer_match, "Footer with totals not found in ledger!"
    total_found = int(footer_match.group(1))
    no_trigger_found = int(footer_match.group(2))

    assert total_found == expected_total, f"Footer total markers mismatch: expected {expected_total}, found {total_found}."
    assert no_trigger_found == expected_no_trigger, f"Footer no-trigger markers mismatch: expected {expected_no_trigger}, found {no_trigger_found}."

    print(f"✅ All {expected_total} ponytail comments are correctly tracked, grouped, and tagged in PONYTAIL-DEBT.md!")

if __name__ == '__main__':
    main()
