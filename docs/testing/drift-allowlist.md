# Test suite drift allowlist

Data file: `scripts/ci/drift_allowlist.txt` (consumed by
`scripts/ci/check_test_drift.py`). This document records the justification
for every entry, as required by the acceptance criteria.

## Policy

1. An entry is only for a suite that **cannot** run in CI. A suite that is
   merely inconvenient belongs in a CI job, not here.
2. Every entry must carry a reason inline (`# reason`) or on the comment
   line directly above. The check fails on unjustified entries.
3. The check fails on **stale** entries (CI executes the path again) and
   **dangling** entries (path no longer exists), so the allowlist cannot
   become the next silent blind spot. Delete such entries.

## Entries

### `tests/python/integration/test_gallery_contract.py`

ignores it explicitly.

### `tests/python/integration/test_gallery_plugin_api.py`

`python -c ...` workers; python-ci.yml integration-tests ignores it.

### `tests/python/integration/test_gallery_real_e2e.py`

python-ci.yml integration-tests ignores it explicitly.

### `tests/test_child.py`

runtime. Fix the patch target before moving this into a CI job.

### `tests/test_examples_startup.py`

window; needs a display and a built example set, and is timing sensitive.
