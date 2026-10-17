#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Full frontmatter/schema parsing, with no third-party parser dependency."""
import json
from pathlib import Path
import unittest
from skill_frontmatter import validate_skill


class SkillFrontmatterTests(unittest.TestCase):
    def test_actual_portable_skill_parses_as_standard_json_subset_yaml(self):
        data = (Path(__file__).resolve().parent.parent / "skills/regressee/SKILL.md").read_bytes()
        parsed = validate_skill(data)
        self.assertEqual(parsed["name"], "regressee")
        self.assertIn("CLI:", parsed["description"])

    def test_legacy_unquoted_colon_and_malformed_or_ambiguous_documents_refuse(self):
        bad = [b"---\nname: regressee\ndescription: CLI: authorized\n---\n# Regressee\n",
               b'---\n{"name":"regressee","name":"regressee","description":"x"}\n---\n# Body',
               b'---\n{"name":"regressee","description":NaN}\n---\n# Body',
               b'---\n{"name":"regressee","description":"x"}\n# Missing closing',
               b'---\n{"name":"regressee","description":"x"}\n---\n',
               b'---\n["name","description"]\n---\n# Body']
        for data in bad:
            with self.subTest(data=data), self.assertRaises(ValueError):
                validate_skill(data)
        for description in (None, [], 3, "", "x" * 1025, "control\nline", "\ud800"):
            data = ("---\n" + json.dumps({"name": "regressee", "description": description}) + "\n---\n# Body").encode()
            with self.subTest(description=description), self.assertRaises(ValueError):
                validate_skill(data)
        with self.assertRaises(ValueError):
            validate_skill(("---\n" + "[" * 2000 + "0" + "]" * 2000 + "\n---\n# Body").encode())


if __name__ == "__main__":
    unittest.main()
