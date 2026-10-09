#![cfg(feature = "emoji")]

use strict_test_support::{ComparisonFailure, PredicateFailure, ensure_eq, ensure_that};
use unicode_properties::{EmojiStatus, UnicodeEmoji};

/// Scalar and the complete public emoji status/predicate observations.
type EmojiCase = (char, EmojiStatus, bool, bool, bool);

/// Every scalar representable by one byte retains its supported status class.
#[test]
fn all_ascii_are_either_nonemoji_or_emojiother()
-> Result<(), PredicateFailure<Vec<(char, EmojiStatus)>>> {
    let observations = (u8::MIN..=u8::MAX)
        .map(|byte| {
            let scalar = char::from(byte);
            (scalar, scalar.emoji_status())
        })
        .collect::<Vec<_>>();
    ensure_that(
        observations,
        "8-bit characters retain their supported emoji status classes",
        |observations| {
            observations.iter().all(|(_, status)| {
                matches!(
                    status,
                    EmojiStatus::NonEmoji
                        | EmojiStatus::EmojiOther
                        | EmojiStatus::EmojiOtherAndEmojiComponent
                )
            })
        },
    )
    .map(drop)
}

/// Preserve the original crab status and all three public predicate outcomes.
#[test]
fn emoji_test() -> Result<(), ComparisonFailure<EmojiCase, EmojiCase>> {
    let scalar = '🦀';
    let observed = (
        scalar,
        scalar.emoji_status(),
        scalar.is_emoji_char(),
        scalar.is_emoji_component(),
        scalar.is_emoji_char_or_emoji_component(),
    );
    ensure_eq(
        observed,
        (scalar, EmojiStatus::EmojiPresentation, true, false, true),
        "crab is an emoji presentation scalar and not an emoji component",
    )
    .map(drop)
}

/// Unicode 18 additions and adjacent unassigned values keep their native status.
#[test]
fn unicode18_emoji_assignments_and_boundaries()
-> Result<(), ComparisonFailure<[EmojiCase; 6], [EmojiCase; 6]>> {
    let expected = [
        (
            '\u{1FA8B}',
            EmojiStatus::EmojiPresentation,
            true,
            false,
            true,
        ),
        (
            '\u{1FA8D}',
            EmojiStatus::EmojiPresentation,
            true,
            false,
            true,
        ),
        (
            '\u{1FACC}',
            EmojiStatus::EmojiPresentation,
            true,
            false,
            true,
        ),
        (
            '\u{1FAF9}',
            EmojiStatus::EmojiPresentationAndModifierBase,
            true,
            false,
            true,
        ),
        (
            '\u{1FAFA}',
            EmojiStatus::EmojiPresentationAndModifierBase,
            true,
            false,
            true,
        ),
        ('\u{1FAFB}', EmojiStatus::NonEmoji, false, false, false),
    ];
    let observed = expected.map(|(scalar, _, _, _, _)| {
        (
            scalar,
            scalar.emoji_status(),
            scalar.is_emoji_char(),
            scalar.is_emoji_component(),
            scalar.is_emoji_char_or_emoji_component(),
        )
    });
    ensure_eq(
        observed,
        expected,
        "Unicode 18 emoji additions and their boundary must retain every property",
    )
    .map(drop)
}

/// Component-only scalars and ordinary characters preserve both property polarities.
#[test]
fn component_only_scalars_are_components_without_being_emoji()
-> Result<(), ComparisonFailure<[EmojiCase; 8], [EmojiCase; 8]>> {
    let expected = [
        (
            '\u{200D}',
            EmojiStatus::NonEmojiButEmojiComponent,
            false,
            true,
            true,
        ),
        (
            '\u{20E3}',
            EmojiStatus::NonEmojiButEmojiComponent,
            false,
            true,
            true,
        ),
        (
            '\u{FE0F}',
            EmojiStatus::NonEmojiButEmojiComponent,
            false,
            true,
            true,
        ),
        (
            '\u{E0020}',
            EmojiStatus::NonEmojiButEmojiComponent,
            false,
            true,
            true,
        ),
        (
            '\u{E007F}',
            EmojiStatus::NonEmojiButEmojiComponent,
            false,
            true,
            true,
        ),
        (
            '#',
            EmojiStatus::EmojiOtherAndEmojiComponent,
            true,
            true,
            true,
        ),
        ('A', EmojiStatus::NonEmoji, false, false, false),
        ('\u{10FFFF}', EmojiStatus::NonEmoji, false, false, false),
    ];
    let observed = expected.map(|(scalar, _, _, _, _)| {
        (
            scalar,
            scalar.emoji_status(),
            scalar.is_emoji_char(),
            scalar.is_emoji_component(),
            scalar.is_emoji_char_or_emoji_component(),
        )
    });
    ensure_eq(
        observed,
        expected,
        "component membership must include component-only scalars and reject noncomponents",
    )
    .map(drop)
}
