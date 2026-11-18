# 0001. Keep the configuration in plain TOML

Status: accepted (2026-11-15)

## Context

The greeter needs a few settings: the default name and the closing line.

## Options

- TOML file in the project
- Environment variables only

## Decision

TOML file in the project: it is versioned and easy to review.

## Consequences

Changes to the defaults show up in code review.
