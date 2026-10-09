//! Compare the public API against independent, complete official Unicode data.

#![cfg(any(feature = "general-category", feature = "emoji"))]

use std::num::{ParseIntError, TryFromIntError};

use strict_test_support::{PredicateFailure, ensure_that};

/// Original record with both native integer parses and its optional property.
type RangeRecord = (
    &'static str,
    Result<usize, ParseIntError>,
    Result<usize, ParseIntError>,
    Option<&'static str>,
);

/// Parse official range records independently of the production generator.
fn range_records(data: &'static str) -> Vec<RangeRecord> {
    data.lines()
        .filter_map(|line| {
            let content = line
                .split_once('#')
                .map_or(line, |(record, _)| record)
                .trim();
            if content.is_empty() {
                return None;
            }
            let fields = content.split_once(';');
            let bounds = fields.map_or(content, |(bounds, _)| bounds).trim();
            let (start, end) = bounds.split_once("..").unwrap_or((bounds, bounds));
            Some((
                line,
                usize::from_str_radix(start, 16),
                usize::from_str_radix(end, 16),
                fields.map(|(_, property)| property.trim()),
            ))
        })
        .collect()
}

#[cfg(feature = "general-category")]
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

/// Normative category abbreviations mapped to the crate's existing public enum.
#[cfg(feature = "general-category")]
const CATEGORY_VALUES: &[(&str, GeneralCategory)] = &[
    ("Lu", GeneralCategory::UppercaseLetter),
    ("Ll", GeneralCategory::LowercaseLetter),
    ("Lt", GeneralCategory::TitlecaseLetter),
    ("Lm", GeneralCategory::ModifierLetter),
    ("Lo", GeneralCategory::OtherLetter),
    ("Mn", GeneralCategory::NonspacingMark),
    ("Mc", GeneralCategory::SpacingMark),
    ("Me", GeneralCategory::EnclosingMark),
    ("Nd", GeneralCategory::DecimalNumber),
    ("Nl", GeneralCategory::LetterNumber),
    ("No", GeneralCategory::OtherNumber),
    ("Pc", GeneralCategory::ConnectorPunctuation),
    ("Pd", GeneralCategory::DashPunctuation),
    ("Ps", GeneralCategory::OpenPunctuation),
    ("Pe", GeneralCategory::ClosePunctuation),
    ("Pi", GeneralCategory::InitialPunctuation),
    ("Pf", GeneralCategory::FinalPunctuation),
    ("Po", GeneralCategory::OtherPunctuation),
    ("Sm", GeneralCategory::MathSymbol),
    ("Sc", GeneralCategory::CurrencySymbol),
    ("Sk", GeneralCategory::ModifierSymbol),
    ("So", GeneralCategory::OtherSymbol),
    ("Zs", GeneralCategory::SpaceSeparator),
    ("Zl", GeneralCategory::LineSeparator),
    ("Zp", GeneralCategory::ParagraphSeparator),
    ("Cc", GeneralCategory::Control),
    ("Cf", GeneralCategory::Format),
    ("Cs", GeneralCategory::Surrogate),
    ("Co", GeneralCategory::PrivateUse),
    ("Cn", GeneralCategory::Unassigned),
];

/// One scalar, actual category, and native conversion/expected-category outcome.
#[cfg(feature = "general-category")]
type CategoryObservation = (
    char,
    GeneralCategory,
    Result<Option<GeneralCategory>, TryFromIntError>,
);

/// Complete version, parsed oracle records, and scalar observations.
#[cfg(feature = "general-category")]
type CategoryEvidence = ((u64, u64, u64), Vec<RangeRecord>, Vec<CategoryObservation>);

/// Every valid scalar must agree with the official extracted category table.
#[cfg(feature = "general-category")]
#[test]
fn categories_match_official_unicode18_for_every_scalar()
-> Result<(), PredicateFailure<CategoryEvidence>> {
    let records = range_records(include_str!(
        "data/unicode/18.0.0/extracted/DerivedGeneralCategory.txt"
    ));
    let mut expected = vec![None; 0x110000];
    for (_, start, end, property) in &records {
        let category = CATEGORY_VALUES
            .iter()
            .find(|(name, _)| Some(*name) == *property)
            .map(|(_, category)| *category);
        if let (Ok(start), Ok(end), Some(category)) = (start, end, category) {
            if let Some(range) = expected.get_mut(*start..=*end) {
                range.fill(Some(category));
            }
        }
    }
    let observations: Vec<CategoryObservation> = (0..=0x10FFFF)
        .filter_map(char::from_u32)
        .map(|scalar| {
            (
                scalar,
                scalar.general_category(),
                usize::try_from(u32::from(scalar))
                    .map(|index| expected.get(index).copied().flatten()),
            )
        })
        .collect();
    ensure_that(
        (unicode_properties::UNICODE_VERSION, records, observations),
        "every scalar category must match the complete official Unicode 18 oracle",
        |(version, records, observations)| {
            *version == (18, 0, 0)
                && records.iter().all(|(_, start, end, property)| {
                    matches!((start, end), (Ok(start), Ok(end)) if start <= end && *end < 0x110000)
                        && CATEGORY_VALUES.iter().any(|(name, _)| Some(*name) == *property)
                })
                && observations.iter().all(|(_, actual, expected)| {
                    matches!(expected, Ok(Some(category)) if category == actual)
                })
        },
    )
    .map(drop)
}

#[cfg(feature = "emoji")]
use unicode_properties::{EmojiStatus, UnicodeEmoji};

/// Property column order in the official data, including its unsupported view.
#[cfg(feature = "emoji")]
const EMOJI_PROPERTIES: [&str; 6] = [
    "Emoji",
    "Emoji_Presentation",
    "Emoji_Modifier",
    "Emoji_Modifier_Base",
    "Emoji_Component",
    "Extended_Pictographic",
];

/// The five binary-property meanings documented for each existing status.
#[cfg(feature = "emoji")]
const STATUS_PROPERTIES: &[(EmojiStatus, [bool; 5])] = &[
    (EmojiStatus::NonEmoji, [false, false, false, false, false]),
    (
        EmojiStatus::NonEmojiButEmojiComponent,
        [false, false, false, false, true],
    ),
    (
        EmojiStatus::EmojiPresentation,
        [true, true, false, false, false],
    ),
    (
        EmojiStatus::EmojiModifierBase,
        [true, false, false, true, false],
    ),
    (
        EmojiStatus::EmojiPresentationAndModifierBase,
        [true, true, false, true, false],
    ),
    (EmojiStatus::EmojiOther, [true, false, false, false, false]),
    (
        EmojiStatus::EmojiPresentationAndEmojiComponent,
        [true, true, false, false, true],
    ),
    (
        EmojiStatus::EmojiPresentationAndModifierAndEmojiComponent,
        [true, true, true, false, true],
    ),
    (
        EmojiStatus::EmojiOtherAndEmojiComponent,
        [true, false, false, false, true],
    ),
];

/// Native status, its documented properties, and all three public predicates.
#[cfg(feature = "emoji")]
type EmojiObservation = (
    char,
    EmojiStatus,
    Option<[bool; 5]>,
    [bool; 3],
    Result<Option<[bool; 5]>, TryFromIntError>,
);

/// Complete version, parsed oracle records, and scalar emoji observations.
#[cfg(feature = "emoji")]
type EmojiEvidence = ((u64, u64, u64), Vec<RangeRecord>, Vec<EmojiObservation>);

/// Every scalar's status and predicates must agree with the official properties.
#[cfg(feature = "emoji")]
#[test]
fn emoji_matches_official_unicode18_for_every_scalar() -> Result<(), PredicateFailure<EmojiEvidence>>
{
    let records = range_records(include_str!("data/unicode/18.0.0/emoji/emoji-data.txt"));
    let mut expected = vec![[false; 5]; 0x110000];
    for (_, start, end, property) in &records {
        let column = EMOJI_PROPERTIES
            .iter()
            .position(|name| Some(*name) == *property);
        if let (Ok(start), Ok(end), Some(column)) = (start, end, column) {
            if let Some(range) = expected.get_mut(*start..=*end) {
                for properties in range {
                    if let Some(value) = properties.get_mut(column) {
                        *value = true;
                    }
                }
            }
        }
    }
    let observations: Vec<EmojiObservation> = (0..=0x10FFFF)
        .filter_map(char::from_u32)
        .map(|scalar| {
            let status = scalar.emoji_status();
            (
                scalar,
                status,
                STATUS_PROPERTIES
                    .iter()
                    .find(|(candidate, _)| *candidate == status)
                    .map(|(_, properties)| *properties),
                [
                    scalar.is_emoji_char(),
                    scalar.is_emoji_component(),
                    scalar.is_emoji_char_or_emoji_component(),
                ],
                usize::try_from(u32::from(scalar)).map(|index| expected.get(index).copied()),
            )
        })
        .collect();
    ensure_that(
        (unicode_properties::UNICODE_VERSION, records, observations),
        "every scalar emoji status and predicate must match official Unicode 18 binary properties",
        |(version, records, observations)| {
            *version == (18, 0, 0)
                && records.iter().all(|(_, start, end, property)| {
                    matches!((start, end), (Ok(start), Ok(end)) if start <= end && *end < 0x110000)
                        && EMOJI_PROPERTIES.iter().any(|name| Some(*name) == *property)
                })
                && observations
                    .iter()
                    .all(|(_, _, status_properties, actual, expected)| {
                        if let Ok(Some(properties)) = expected {
                            let [emoji, .., component] = *properties;
                            *status_properties == Some(*properties)
                                && *actual == [emoji, component, emoji || component]
                        } else {
                            false
                        }
                    })
        },
    )
    .map(drop)
}
