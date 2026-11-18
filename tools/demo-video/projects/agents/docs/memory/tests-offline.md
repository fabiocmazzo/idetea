---
name: tests-offline
description: the tests must not use the network; CI runs offline
type: constraint
updated: 2026-11-16
---
The tests run in CI without network access. Use the fixtures in `tests/fixtures/`.

**Why:** the CI runner has no internet access.
