---
name: e2e-self-iteration
description: "Integrates ProofShot visual proof recording with the self-improvement workflow. Use when: (1) E2E tests fail or detect visual regressions, (2) Agent needs to verify UI changes it made, (3) Running self-iteration loops (build → test → fix → retest), (4) Preparing PR proof artifacts for review."
metadata:
---

# E2E Self-Iteration Skill

Combines **ProofShot** (visual proof recording) and **agent-browser** (headless browser control) with the self-improvement loop for automated E2E testing, visual regression detection, and iterative bug fixing.

## Quick Reference

| Situation | Action |
|-----------|--------|
| Need to verify UI changes | `vx just e2e-proofshot` |
| Found visual regression | Log to `.learnings/ERRORS.md`, fix, re-run |
| Self-iteration loop | `vx just e2e-iterate` |
| PR needs proof | `vx just e2e-pr` |
| Explore UI elements | `vx just e2e-snapshot` |
| Compare against baseline | `vx just e2e-diff` |

## Prerequisites

```bash
# Install ProofShot + agent-browser
vx just e2e-install
```

## Workflows

### 1. Quick Visual Verification

After making UI changes, verify them visually:

```bash
vx just e2e-start          # Build + start Gallery with CDP
vx just e2e-snapshot        # See interactive elements
vx just e2e-screenshot      # Capture annotated screenshot
vx just e2e-stop            # Cleanup
```

### 2. Full ProofShot Session

Record a complete verification session with video + artifacts:

```bash
vx just e2e-proofshot       # Full automated flow
```

This will:
1. Pack Gallery (debug mode)
2. Start with CDP enabled
3. Record ProofShot session
4. Capture snapshots and screenshots
5. Run Playwright CDP tests
6. Stop and generate artifacts

### 3. Self-Iteration Loop

When fixing bugs or implementing features, iterate:

```bash
# Cycle: build → start → verify → fix → repeat
vx just e2e-iterate

# Review artifacts after each iteration
ls proofshot-artifacts/
```

### 4. agent-browser Primitives

Direct browser control commands:

```bash
# Element discovery
vx just e2e-snapshot

# Navigation
vx just e2e-open "http://localhost:5173/settings"

# Interaction
vx just e2e-click "@e5"
vx just e2e-fill "@e2" "test value"

# Screenshot
vx just e2e-screenshot
```

### 5. Visual Regression Testing

Compare current state against baseline:

```bash
vx just e2e-diff
```

### 6. PR Proof Upload

Attach visual proof to a Pull Request:

```bash
vx just e2e-pr              # Auto-detect PR
vx just e2e-pr 42           # Specific PR number
```

## Self-Iteration Protocol

When the agent detects issues during E2E testing:

1. **Capture**: Save screenshot of the issue
2. **Log**: Create entry in `.learnings/ERRORS.md`
   ```markdown
   ## [ERR-YYYYMMDD-XXX] e2e_visual_regression

   **Logged**: ISO-8601 timestamp
   **Priority**: high
   **Status**: pending
   **Area**: frontend

   ### Summary
   [Description of what's wrong visually]

   ### Error
   [Console errors if any]

   ### Context
   - Screenshot: proofshot-artifacts/<session>/step-*.png
   - Page: [URL or route]
   - Expected: [what it should look like]
   - Actual: [what it looks like]

   ### Suggested Fix
   [Specific code change needed]
   ```
3. **Fix**: Apply the code change
4. **Re-verify**: Run `vx just e2e-iterate` again
5. **Resolve**: Update status to `resolved` in `.learnings/ERRORS.md`

## Integration with CI

```bash
# CI pipeline step
vx just e2e-ci

# Upload proof to PR after CI tests
vx just e2e-pr
```

## Artifact Structure

```
proofshot-artifacts/
└── <timestamp>/
    ├── session.webm          # Full session video
    ├── viewer.html           # Interactive viewer
    ├── SUMMARY.md            # Markdown report
    ├── step-*.png            # Key screenshots
    ├── session-log.json      # Action timeline
    ├── server.log            # Server stdout/stderr
    └── console-output.log    # Browser console
```
