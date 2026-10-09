#![cfg(feature = "general-category")]

use strict_test_support::{ComparisonFailure, ensure_eq};
use unicode_properties::{GeneralCategory, GeneralCategoryGroup, UnicodeGeneralCategory};

/// Scalar, native category/group, and cased-letter membership.
type CategoryCase = (char, GeneralCategory, GeneralCategoryGroup, bool);

/// Preserve the original category, group, and cased-letter examples.
#[test]
fn general_category_test() -> Result<(), ComparisonFailure<[CategoryCase; 4], [CategoryCase; 4]>> {
    let expected = [
        (
            'A',
            GeneralCategory::UppercaseLetter,
            GeneralCategoryGroup::Letter,
            true,
        ),
        (
            ' ',
            GeneralCategory::SpaceSeparator,
            GeneralCategoryGroup::Separator,
            false,
        ),
        (
            '一',
            GeneralCategory::OtherLetter,
            GeneralCategoryGroup::Letter,
            false,
        ),
        (
            '🦀',
            GeneralCategory::OtherSymbol,
            GeneralCategoryGroup::Symbol,
            false,
        ),
    ];
    let observed = expected.map(|(scalar, _, _, _)| {
        (
            scalar,
            scalar.general_category(),
            scalar.general_category_group(),
            scalar.is_letter_cased(),
        )
    });
    ensure_eq(
        observed,
        expected,
        "the original category and cased-letter examples retain their complete outcomes",
    )
    .map(drop)
}

/// New assignments, range endpoints, and default categories follow Unicode 18.
#[test]
fn unicode18_categories_and_unassigned_boundaries()
-> Result<(), ComparisonFailure<[CategoryCase; 9], [CategoryCase; 9]>> {
    let expected = [
        (
            '\u{12550}',
            GeneralCategory::LetterNumber,
            GeneralCategoryGroup::Number,
            false,
        ),
        (
            '\u{18E00}',
            GeneralCategory::OtherLetter,
            GeneralCategoryGroup::Letter,
            false,
        ),
        (
            '\u{19191}',
            GeneralCategory::OtherLetter,
            GeneralCategoryGroup::Letter,
            false,
        ),
        (
            '\u{19192}',
            GeneralCategory::Unassigned,
            GeneralCategoryGroup::Other,
            false,
        ),
        (
            '\u{191A0}',
            GeneralCategory::OtherLetter,
            GeneralCategoryGroup::Letter,
            false,
        ),
        (
            '\u{1FA8B}',
            GeneralCategory::OtherSymbol,
            GeneralCategoryGroup::Symbol,
            false,
        ),
        (
            '\u{E000}',
            GeneralCategory::PrivateUse,
            GeneralCategoryGroup::Other,
            false,
        ),
        (
            '\u{10FFFF}',
            GeneralCategory::Unassigned,
            GeneralCategoryGroup::Other,
            false,
        ),
        (
            '\u{01C5}',
            GeneralCategory::TitlecaseLetter,
            GeneralCategoryGroup::Letter,
            true,
        ),
    ];
    let observed = expected.map(|(scalar, _, _, _)| {
        (
            scalar,
            scalar.general_category(),
            scalar.general_category_group(),
            scalar.is_letter_cased(),
        )
    });
    ensure_eq(
        observed,
        expected,
        "Unicode 18 assignments, gaps, and category groups must agree",
    )
    .map(drop)
}
