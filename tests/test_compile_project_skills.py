"""Exercise source ownership, freshness failures, and relocated links."""

import importlib.util
from pathlib import Path
import re
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("compiler", ROOT / "tools/compile_project_skills.py")
compiler = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(compiler)


class CompileProjectSkillsTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="project-skills-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        for spec in compiler.SPECS:
            path = self.root / spec["source"]
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text((ROOT / spec["source"]).read_text(encoding="utf-8"), encoding="utf-8")

    def test_generation_is_repeatable_and_fresh(self):
        compiler.compile_skills(self.root)
        paths = [compiler.render(self.root, spec)[0] for spec in compiler.SPECS]
        before = [path.read_bytes() for path in paths]
        compiler.compile_skills(self.root)
        self.assertEqual(before, [path.read_bytes() for path in paths])
        self.assertEqual([], compiler.compile_skills(self.root, check=True))

    def test_pr_skill_is_not_generated_or_checked_for_freshness(self):
        path = self.root / ".agents/skills/project-prepare-pr/SKILL.md"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("Manually maintained PR procedure.\n", encoding="utf-8")
        compiler.compile_skills(self.root)
        self.assertEqual([], compiler.compile_skills(self.root, check=True))
        self.assertEqual("Manually maintained PR procedure.\n", path.read_text(encoding="utf-8"))

    def test_changed_procedure_is_stale_and_check_does_not_write(self):
        compiler.compile_skills(self.root)
        spec = compiler.SPECS[0]
        source = self.root / spec["source"]
        source.write_text(source.read_text(encoding="utf-8").replace(
            spec["heading"] + "\n", spec["heading"] + "\n\nNew required observation.\n", 1
        ), encoding="utf-8")
        path, _ = compiler.render(self.root, spec)
        before = path.read_bytes()
        self.assertEqual([path.relative_to(self.root).as_posix()], compiler.compile_skills(self.root, check=True))
        self.assertEqual(before, path.read_bytes())
        compiler.compile_skills(self.root)
        self.assertEqual([], compiler.compile_skills(self.root, check=True))

    def test_missing_and_manually_edited_outputs_are_detected(self):
        self.assertEqual(len(compiler.SPECS), len(compiler.compile_skills(self.root, check=True)))
        compiler.compile_skills(self.root)
        path, _ = compiler.render(self.root, compiler.SPECS[0])
        path.write_text(path.read_text(encoding="utf-8") + "Manual procedure change.\n", encoding="utf-8")
        self.assertEqual([path.relative_to(self.root).as_posix()], compiler.compile_skills(self.root, check=True))

    def test_unrelated_document_change_does_not_change_generated_steps(self):
        compiler.compile_skills(self.root)
        source = self.root / compiler.SPECS[0]["source"]
        source.write_text(source.read_text(encoding="utf-8") + "\n## Unrelated notes\n\nUnrelated change.\n", encoding="utf-8")
        self.assertEqual([], compiler.compile_skills(self.root, check=True))

    def test_invalid_source_does_not_overwrite_output(self):
        compiler.compile_skills(self.root)
        first_path, _ = compiler.render(self.root, compiler.SPECS[0])
        first_path.write_text("Existing content must survive.\n", encoding="utf-8")
        spec = compiler.SPECS[0]
        source = self.root / spec["source"]
        original = source.read_text(encoding="utf-8")
        for text in (original.replace(spec["heading"], "Missing section"),
                     original + "\n## " + spec["heading"] + "\n\nAmbiguous section.\n"):
            source.write_text(text, encoding="utf-8")
            with self.assertRaises(ValueError):
                compiler.compile_skills(self.root)
            self.assertEqual("Existing content must survive.\n", first_path.read_text(encoding="utf-8"))

    def test_relative_and_fragment_links_keep_their_destinations(self):
        source = self.root / "docs/testing.md"
        destination = self.root / ".agents/skills/example/SKILL.md"
        text = "[Local](#checks) [Other](../CONTRIBUTING.md#workflow) [Web](https://example.com/a#b)"
        relocated = compiler.relocate_links(text, source, destination)
        self.assertEqual(
            "[Local](../../../docs/testing.md#checks) [Other](../../../CONTRIBUTING.md#workflow) [Web](https://example.com/a#b)", relocated
        )
        for spec in compiler.SPECS:
            dest, content = compiler.render(ROOT, spec)
            for link in re.findall(r"\[[^\]\n]+\]\(([^)\n]+)\)", content):
                if "://" not in link:
                    self.assertTrue((dest.parent / link.split("#")[0]).resolve().exists(), link)

    def test_fenced_headings_do_not_end_a_procedure(self):
        text = "## Procedure\n\n1. Start.\n\n```markdown\n## Example\n```\n\n2. Finish.\n\n## Next\nOther.\n"
        result = compiler.section(text, "Procedure")
        self.assertIn("2. Finish.", result)
        self.assertNotIn("Other.", result)


if __name__ == "__main__":
    unittest.main()
