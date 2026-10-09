<!-- Do not edit; generated file. -->

# unicode-properties

[![Build Status](https://github.com/strict-rs/unicode-properties/actions/workflows/rust.yml/badge.svg?branch=upgrade)](https://github.com/strict-rs/unicode-properties/actions/workflows/rust.yml)
[![Upstream crates.io release](https://img.shields.io/crates/v/unicode-properties.svg)](https://crates.io/crates/unicode-properties)

Query Unicode `18.0.0` general categories and emoji properties with `#![no_std]`, no allocation, and no runtime dependencies. This fork uses Rust edition `2024`, Rust `1.99.0`, and the strict ecosystem's `upgrade` adoption stage.

## Features and API

| Feature | Public API | Default |
| --- | --- | --- |
| `general-category` | `UnicodeGeneralCategory`, `GeneralCategory`, `GeneralCategoryGroup` | Enabled |
| `emoji` | `UnicodeEmoji`, `EmojiStatus`, emoji selector and sequence helpers | Enabled |

The two features remain independently selectable. `UNICODE_VERSION` is available with every feature selection, including no features, and reports `(18, 0, 0)` as `(u64, u64, u64)`.

`is_emoji_component()` follows the official `Emoji_Component` property, including component-only scalars such as `U+200D`, `U+20E3`, and `U+FE0F`. These scalars have `EmojiStatus::NonEmojiButEmojiComponent`: they are components and sequence participants, while `is_emoji_char()` returns `false`.

## Consume the fork

The package remains `unicode-properties` `0.1.4`; the upstream crates.io release and this Git source have distinct Unicode data. Apply the source override at the consuming workspace root so Cargo also selects it for transitive consumers:

```toml
[patch.crates-io]
unicode-properties = { version = "0.1.4", git = "https://github.com/strict-rs/unicode-properties.git", branch = "upgrade" }
```

Preserve each consumer's feature declarations and let Cargo refresh its lockfile. Consumers that require matching identifier/property releases must use Unicode `18.0.0` identifier data as well; the source override does not remove their version-equality checks.

## Regenerate Unicode data

`scripts/unicode.py` uses Python `3.11` or newer and only the standard library. The complete official release bundle is retained beneath `tests/data/unicode/18.0.0/`:

- `UnicodeData.txt` supplies the generated general-category ranges.
- `emoji/emoji-data.txt` supplies all five binary properties represented by `EmojiStatus`.
- `extracted/DerivedGeneralCategory.txt` supplies an independent official category oracle for Rust tests.
- `ReadMe.txt` identifies the final release; `sources.json` records each official versioned URL and SHA-256 digest.

Regeneration runs offline, verifies every retained input's version and digest, validates the complete properties, and replaces the output atomically. Invalid input or a failed replacement preserves the previous output. A cleanup failure retains both the original write failure and the cleanup failure.

```sh
python3 scripts/unicode.py --output src/tables.rs
just fmt
```

The legacy invocation from `scripts/`, `python3 unicode.py`, still writes `scripts/tables.rs`. Use `--output` to select another destination and `--data-directory` to select a complete versioned input bundle. Progress and failures go to stderr; stdout stays empty during generation.

When updating the release, retrieve complete files from the exact official versioned URLs, retain their copyright/license notices, and explicitly update the source manifest's actual digests. A changed digest records input adoption; it does not replace the independent property comparisons. Never change the reported Unicode version without regenerating both supported property families.

## Verify the upgrade stage

```sh
python3 -m unittest discover -s scripts -p 'test_unicode.py' -v
python3 scripts/unicode.py --output scripts/tables.rs
diff -u src/tables.rs scripts/tables.rs
```

Run `just check -- <FEATURE_ARGS>` and `just test-all -- <FEATURE_ARGS>` for every row below. `test-all` includes integration tests and doctests. The data-driven integration tests compare every valid Rust scalar against the official category and emoji records.

| Selection | `FEATURE_ARGS` |
| --- | --- |
| Default | `--features default` |
| All | `--all-features` |
| None | `--no-default-features` |
| General category alone | `--no-default-features --features general-category` |
| Emoji alone | `--no-default-features --features emoji` |

`just fmt` is the workspace formatter. The `upgrade` workflow exercises the declared Rust baseline, feature matrix, generator failure boundaries, and complete table regeneration. Strict lint, coverage, and mutation adoption belongs to the later `strict` stage.

## License

Code uses `MIT OR Apache-2.0`; Unicode data additionally uses `Unicode-3.0`. The package's complete SPDX expression is `(MIT OR Apache-2.0) AND Unicode-3.0`. See `LICENSE-MIT`, `LICENSE-APACHE`, and `LICENSE-UNICODE`; retained official data keeps its Unicode copyright and terms-of-use notices.
