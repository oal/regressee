#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Validate our portable skill's deliberately JSON-subset YAML 1.2 frontmatter.

JSON mappings are standard YAML, so normal YAML skill hosts can load them. Using
this subset lets source/package checks parse the *entire* document with Python's
stdlib, without a home-grown general YAML parser or an unpinned production dep.
"""
import json
import re


def validate_skill(data):
    if not isinstance(data, bytes) or len(data) > 65536:
        raise ValueError("Skill must be bounded UTF-8 Markdown")
    lines = data.decode("utf-8").splitlines()
    if not lines or lines[0] != "---":
        raise ValueError("Skill frontmatter opening delimiter missing")
    try:
        end = lines.index("---", 1)
    except ValueError:
        raise ValueError("Skill frontmatter closing delimiter missing") from None

    def unique(pairs):
        value = {}
        for key, child in pairs:
            if key in value:
                raise ValueError("Duplicate skill frontmatter key")
            value[key] = child
        return value

    def invalid_constant(_):
        raise ValueError("Non-JSON skill frontmatter constant")

    try:
        value = json.loads("\n".join(lines[1:end]), object_pairs_hook=unique,
                           parse_constant=invalid_constant)
    except RecursionError:
        raise ValueError("Skill frontmatter nesting exceeds parser bounds") from None
    if not isinstance(value, dict) or set(value) != {"name", "description"}:
        raise ValueError("Skill requires exactly name and description")
    if value["name"] != "regressee" or not isinstance(value["description"], str):
        raise ValueError("Skill name/description schema mismatch")
    description = value["description"]
    if not 1 <= len(description) <= 1024 or re.search(r"[\x00-\x1f\x7f-\x9f]", description):
        raise ValueError("Skill description must be nonempty ordinary text")
    description.encode("utf-8", errors="strict")
    if not any(line.startswith("# ") for line in lines[end + 1:]):
        raise ValueError("Skill body is missing")
    return value
