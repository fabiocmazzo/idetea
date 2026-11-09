# Code Signing & Privacy Policy — IDE TEA

Free code signing on Windows is provided by [SignPath.io](https://signpath.io), certificate by the [SignPath Foundation](https://signpath.org).

---

## 1. Code Signing Policy

- **Source Repository:** Official releases of IDE TEA are built exclusively from tagged commits (`v*`) in the public GitHub repository [`https://github.com/fabiocmazzo/idetea`](https://github.com/fabiocmazzo/idetea) under the [Apache License 2.0](LICENSE).
- **Automated Build Pipeline:** All release binaries (`idetea.exe` and `IDE_TEA-*-x86_64-setup.exe` for Windows, `idetea` / `IDE TEA.app` for macOS, and `.tar.gz` / `.AppImage` for Linux) are compiled reproducibly via GitHub Actions (`.github/workflows/release.yml`) on clean GitHub-hosted runners. Local developer builds are never signed with production certificates.
- **Committers, Reviewers, and Approvers:**
  - [Fabio C. Mazzo (`@fabiocmazzo`)](https://github.com/fabiocmazzo) — Author, maintainer, reviewer, and release approver.

---

## 2. Privacy Policy

**This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it.**

Specifically, IDE TEA:
- Contains **no telemetry, analytics, crash reporting, or usage tracking**.
- Requires **no user account, login, or license key**.
- Performs **no automatic background network requests** or silent update checks.
- Accesses the network **only** when the user explicitly triggers an action that requires it (such as `git fetch`, `git pull`, `git push` in the Git panel, or installing agent skills from a Git repository via `idetea skills`).
