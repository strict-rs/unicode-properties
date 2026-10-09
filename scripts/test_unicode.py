"""Behavior tests for release validation, property preparation, and atomic writes."""

import hashlib
import json
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import unicode as generator


def category_record(point, name, category):
    """Create one complete UnicodeData record for a parser-owned scenario."""
    return ";".join((point, name, category, "0", "L", "", "", "", "", "N", "", "", "", "", ""))


class UnicodeGeneratorTests(unittest.TestCase):
    """Exercise real input bundles and the generator's native failure boundaries."""

    def setUp(self):
        self.workspace = tempfile.TemporaryDirectory()
        self.addCleanup(self.workspace.cleanup)
        self.root = Path(self.workspace.name)
        self.output = self.root / "tables.rs"
        self.previous = b"previous complete generated table\n"
        self.output.write_bytes(self.previous)

    def copy_bundle(self):
        """Materialize the original complete input role for controlled mutation."""
        return Path(shutil.copytree(generator.DEFAULT_DATA_DIRECTORY, self.root / "18.0.0"))

    def replace_input(self, bundle, name, text):
        """Change parser input while keeping its manifest digest independently valid."""
        data = text.encode("utf-8")
        (bundle / name).write_bytes(data)
        manifest_path = bundle / "sources.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["files"][name]["sha256"] = hashlib.sha256(data).hexdigest()
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

    def test_prepares_complete_official_unicode18_data(self):
        prepared = generator.prepare_tables(generator.DEFAULT_DATA_DIRECTORY)
        self.assertIn((0x12550, 0x12550, "Nl"), prepared.categories)
        self.assertIn((0x18E00, 0x19191, "Lo"), prepared.categories)
        for point, expected in ((0x200D, "NonEmojiButEmojiComponent"),
                                (0x1FA8B, "EmojiPresentation"),
                                (0x1FAF9, "EmojiPresentationAndModifierBase"),
                                (0x10FFFF, "NonEmoji")):
            with self.subTest(point=point):
                actual = [(start, end, status) for start, end, status in prepared.emoji if start <= point <= end]
                self.assertEqual(len(actual), 1)
                self.assertEqual(actual[0][2], expected)
        self.assertEqual(prepared.emoji[0][0], 0)
        self.assertEqual(prepared.emoji[-1][1], 0x10FFFF)
        self.assertTrue(all(not (start <= 0xDFFF and end >= 0xD800)
                            for start, end, _ in prepared.emoji))

    def test_generates_repeatable_complete_output_and_preserves_mode(self):
        self.output.chmod(0o640)
        phases = []
        generated = generator.generate(generator.DEFAULT_DATA_DIRECTORY, self.output, phases.append)
        first = self.output.read_bytes()
        self.assertEqual(first, generated.rendered.encode("utf-8"))
        self.assertEqual(stat.S_IMODE(self.output.stat().st_mode), 0o640)
        self.assertEqual(phases, ["validate official input bundle", "render complete property tables",
                                  f"replace {generator.display_path(self.output)}"])
        repeated = generator.generate(generator.DEFAULT_DATA_DIRECTORY, self.output)
        self.assertEqual(self.output.read_bytes(), first)
        self.assertEqual(repeated.prepared, generated.prepared)

    def test_rejects_stale_release_before_output_mutation(self):
        bundle = self.copy_bundle()
        path = bundle / "sources.json"
        manifest = json.loads(path.read_text(encoding="utf-8"))
        manifest["unicode_version"] = "17.0.0"
        path.write_text(json.dumps(manifest), encoding="utf-8")
        with self.assertRaises(generator.GenerationError) as failure:
            generator.generate(bundle, self.output)
        self.assertEqual((failure.exception.code, failure.exception.expected, failure.exception.received),
                         ("input-version", "18.0.0", "17.0.0"))
        self.assertEqual(self.output.read_bytes(), self.previous)

    def test_rejects_partial_or_misdirected_input_bundle(self):
        for defect in ("missing", "source"):
            with self.subTest(defect=defect):
                bundle = Path(shutil.copytree(generator.DEFAULT_DATA_DIRECTORY, self.root / defect))
                path = bundle / "sources.json"
                manifest = json.loads(path.read_text(encoding="utf-8"))
                if defect == "missing":
                    del manifest["files"]["UnicodeData.txt"]
                else:
                    manifest["files"]["UnicodeData.txt"]["url"] = "https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt"
                path.write_text(json.dumps(manifest), encoding="utf-8")
                with self.assertRaises(generator.GenerationError) as failure:
                    generator.generate(bundle, self.output)
                self.assertEqual(failure.exception.code, "input-files" if defect == "missing" else "input-source")
                self.assertEqual(self.output.read_bytes(), self.previous)

    def test_rejects_changed_payload_without_rewriting_its_digest(self):
        bundle = self.copy_bundle()
        (bundle / "UnicodeData.txt").write_bytes(b"changed input\n")
        with self.assertRaises(generator.GenerationError) as failure:
            generator.generate(bundle, self.output)
        self.assertEqual(failure.exception.code, "input-digest")
        self.assertNotEqual(failure.exception.expected, failure.exception.received)
        self.assertEqual(self.output.read_bytes(), self.previous)

    def test_rejects_wrong_property_version_even_with_matching_digest(self):
        bundle = self.copy_bundle()
        text = (bundle / "emoji/emoji-data.txt").read_text(encoding="utf-8")
        self.replace_input(bundle, "emoji/emoji-data.txt", text.replace("# Version: 18.0.0", "# Version: 17.0.0"))
        with self.assertRaises(generator.GenerationError) as failure:
            generator.generate(bundle, self.output)
        self.assertEqual(failure.exception.code, "property-version")
        self.assertEqual(self.output.read_bytes(), self.previous)

    def test_accepts_a_materialized_complete_release_bundle(self):
        bundle = self.copy_bundle()
        generated = generator.generate(bundle, self.output)
        self.assertEqual(generated.prepared, generator.prepare_tables(generator.DEFAULT_DATA_DIRECTORY))
        self.assertEqual(self.output.read_text(encoding="utf-8"), generated.rendered)

    def test_rejects_nonobject_source_manifest(self):
        bundle = self.copy_bundle()
        (bundle / "sources.json").write_text("[]", encoding="utf-8")
        with self.assertRaises(generator.GenerationError) as failure:
            generator.generate(bundle, self.output)
        self.assertEqual((failure.exception.code, failure.exception.received), ("input-manifest", []))
        self.assertEqual(self.output.read_bytes(), self.previous)

    def test_retains_native_json_failure_and_manifest_identity(self):
        bundle = self.copy_bundle()
        path = bundle / "sources.json"
        path.write_text("{invalid", encoding="utf-8")
        with self.assertRaises(generator.GenerationError) as failure:
            generator.generate(bundle, self.output)
        self.assertEqual((failure.exception.code, failure.exception.source), ("input-manifest-json", path))
        self.assertIsInstance(failure.exception.received, json.JSONDecodeError)
        self.assertIs(failure.exception.__cause__, failure.exception.received)
        self.assertEqual(failure.exception.received.doc, "{invalid")
        self.assertEqual(self.output.read_bytes(), self.previous)

    def test_retains_native_missing_input_failure_before_writing(self):
        bundle = self.copy_bundle()
        path = bundle / "UnicodeData.txt"
        path.unlink()
        with self.assertRaises(generator.GenerationError) as failure:
            generator.generate(bundle, self.output)
        self.assertEqual((failure.exception.code, failure.exception.source), ("input-read", path))
        self.assertIsInstance(failure.exception.received, FileNotFoundError)
        self.assertIs(failure.exception.__cause__, failure.exception.received)
        self.assertEqual(self.output.read_bytes(), self.previous)

    def test_retains_invalid_utf8_payload_in_the_native_decoder_failure(self):
        data = b"\xff"
        path = Path("UnicodeData.txt")
        with self.assertRaises(generator.GenerationError) as failure:
            generator.decode_input(data, path)
        self.assertEqual((failure.exception.code, failure.exception.source), ("input-encoding", path))
        self.assertIsInstance(failure.exception.received, UnicodeDecodeError)
        self.assertIs(failure.exception.received.object, data)
        self.assertIs(failure.exception.__cause__, failure.exception.received)

    def test_rejects_inconsistent_release_and_oracle_metadata(self):
        for name, text, code in (
            ("ReadMe.txt", "draft data files for version 18.0.0\n", "release-metadata"),
            ("extracted/DerivedGeneralCategory.txt", "# DerivedGeneralCategory-17.0.0.txt\n", "property-version"),
        ):
            with self.subTest(name=name):
                bundle = Path(shutil.copytree(generator.DEFAULT_DATA_DIRECTORY, self.root / code))
                self.replace_input(bundle, name, text)
                with self.assertRaises(generator.GenerationError) as failure:
                    generator.generate(bundle, self.output)
                self.assertEqual(failure.exception.code, code)
                self.assertEqual(self.output.read_bytes(), self.previous)

    def test_rejects_surrogate_category_records_before_rendering(self):
        for point, category in (("D800", "Lo"), ("0041", "Cs")):
            with self.subTest(point=point, category=category):
                with self.assertRaises(generator.GenerationError) as failure:
                    generator.load_general_category_properties(category_record(point, "INVALID", category),
                                                               Path("UnicodeData.txt"))
                self.assertEqual(failure.exception.code, "category-surrogate")

    def test_loads_singletons_and_paired_category_ranges(self):
        data = "\n".join((category_record("0041", "LATIN CAPITAL LETTER A", "Lu"),
                          category_record("18E00", "<Jurchen Character, First>", "Lo"),
                          category_record("19191", "<Jurchen Character, Last>", "Lo")))
        self.assertEqual(generator.load_general_category_properties(data, Path("UnicodeData.txt")),
                         [(0x41, 0x41, "Lu"), (0x18E00, 0x19191, "Lo")])

    def test_rejects_malformed_unpaired_and_overlapping_categories(self):
        first = category_record("18E00", "<Jurchen Character, First>", "Lo")
        scenarios = (
            ("malformed", "0041;LETTER;Lu", "category-record"),
            ("unpaired", first, "category-completeness"),
            ("name", first + "\n" + category_record("19191", "<Different, Last>", "Lo"), "category-pair"),
            ("category", first + "\n" + category_record("19191", "<Jurchen Character, Last>", "Nl"), "category-pair"),
            ("overlap", category_record("0041", "A", "Lu") + "\n" + category_record("0041", "B", "Ll"), "range-order"),
            ("codepoint", category_record("110000", "OUTSIDE", "Lo"), "code-point"),
            ("hex", category_record("+0041", "INVALID HEX", "Lu"), "code-point"),
        )
        for name, data, code in scenarios:
            with self.subTest(name=name):
                with self.assertRaises(generator.GenerationError) as failure:
                    generator.load_general_category_properties(data, Path("UnicodeData.txt"))
                self.assertEqual(failure.exception.code, code)

    def test_rejects_malformed_emoji_ranges_and_missing_properties(self):
        cases = (("0041..0030 ; Emoji", "range-order"),
                 ("0041 ; Unknown_Property", "emoji-record"),
                 ("0041..0042..0043 ; Emoji", "emoji-range"),
                 ("0041 ; Emoji\n0041 ; Emoji", "range-order"),
                 ("0041 ; Emoji", "emoji-property"))
        for text, code in cases:
            with self.subTest(text=text):
                with self.assertRaises(generator.GenerationError) as failure:
                    generator.load_emoji_properties(text, Path("emoji-data.txt"))
                self.assertEqual(failure.exception.code, code)

    def test_rejects_unrepresentable_emoji_combinations_and_surrogates(self):
        properties = {name: [] for name in generator.EMOJI_PROPERTIES}
        properties["Emoji_Modifier"] = [(0x41, 0x41)]
        with self.assertRaises(generator.GenerationError) as failure:
            generator.build_emoji_table(properties, Path("emoji-data.txt"))
        self.assertEqual(failure.exception.code, "emoji-combination")
        self.assertEqual(failure.exception.received, (0x41, 0x41, frozenset(("Emoji_Modifier",))))
        properties["Emoji_Modifier"] = [(0xD800, 0xDFFF)]
        with self.assertRaises(generator.GenerationError) as failure:
            generator.build_emoji_table(properties, Path("emoji-data.txt"))
        self.assertEqual(failure.exception.code, "emoji-surrogate")

    def test_parser_rejection_preserves_real_previous_output(self):
        bundle = self.copy_bundle()
        self.replace_input(bundle, "UnicodeData.txt", category_record("18E00", "<Jurchen Character, First>", "Lo"))
        phases = []
        with self.assertRaises(generator.GenerationError) as failure:
            generator.generate(bundle, self.output, phases.append)
        self.assertEqual(failure.exception.code, "category-completeness")
        self.assertEqual(phases, ["validate official input bundle"])
        self.assertEqual(self.output.read_bytes(), self.previous)

    def test_replace_failure_preserves_previous_output_and_cleans_temporary(self):
        original = OSError("replacement failed")
        with patch.object(generator.os, "replace", side_effect=original):
            with self.assertRaises(OSError) as failure:
                generator.write_atomic(self.output, "replacement table\n")
        self.assertIs(failure.exception, original)
        self.assertEqual(self.output.read_bytes(), self.previous)
        self.assertEqual(list(self.root.iterdir()), [self.output])

    def test_retains_both_write_and_cleanup_failures(self):
        original = OSError("replacement failed")
        cleanup = OSError("cleanup failed")
        with patch.object(generator.os, "replace", side_effect=original), patch.object(Path, "unlink", side_effect=cleanup):
            with self.assertRaises(ExceptionGroup) as failure:
                generator.write_atomic(self.output, "replacement table\n")
        self.assertEqual(failure.exception.exceptions, (original, cleanup))
        self.assertEqual(failure.exception.output, self.output)
        self.assertTrue(failure.exception.temporary.is_file())
        self.assertEqual(failure.exception.temporary.parent, self.output.parent)
        self.assertEqual(self.output.read_bytes(), self.previous)

    def test_legacy_invocation_and_help_keep_their_output_contracts(self):
        script = generator.REPOSITORY_ROOT / "scripts/unicode.py"
        help_result = subprocess.run([sys.executable, str(script), "--help"], cwd=self.root,
                                     capture_output=True, text=True, check=False)
        self.assertEqual(help_result.returncode, 0)
        self.assertTrue(help_result.stdout)
        self.assertEqual(help_result.stderr, "")
        self.assertEqual(self.output.read_bytes(), self.previous)
        generated = subprocess.run([sys.executable, str(script)], cwd=self.root,
                                   capture_output=True, text=True, check=False)
        self.assertEqual(generated.returncode, 0)
        self.assertEqual(generated.stdout, "")
        self.assertEqual(self.output.read_text(encoding="utf-8"),
                         generator.render_tables(generator.prepare_tables(generator.DEFAULT_DATA_DIRECTORY)))


if __name__ == "__main__":
    unittest.main()
