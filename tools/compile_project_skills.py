"""Generate document-owned project skills, or check their freshness."""

import argparse
import os
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[1]
SPECS = (
    {
        "name": "pfd-validate-presentation",
        "title": "Validate PFD Presentation",
        "description": "Validate PFD geometry, SVG layers, rendering lifecycle, or browser module integration. Use for presentation changes and visual regressions, not prose-only edits or certification of real-phone acceptance.",
        "goal": "Produce traceable presentation-validation results with unresolved failures and device-evidence limits stated explicitly.",
        "source": "docs/testing.md",
        "heading": "PFD presentation validation procedure",
    },
)


def section(text, title):
    """Select one named section, ignoring headings in fenced examples."""
    lines = text.splitlines(keepends=True)
    headings = []
    fence = None
    for index, line in enumerate(lines):
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker:
            token = marker[1]
            if fence is None:
                fence = token
            elif token[0] == fence[0] and len(token) >= len(fence):
                fence = None
            continue
        if fence is None:
            match = re.match(r"^(#{1,6}) (.+?)\s*$", line)
            if match:
                headings.append((index, len(match[1]), match[2]))
    matches = [item for item in headings if item[2] == title]
    if len(matches) != 1:
        raise ValueError(f"Expected exactly one section: {title}")
    start, level, _ = matches[0]
    end = next((i for i, depth, _ in headings if i > start and depth <= level), len(lines))
    return "".join(lines[start + 1:end]).strip()


def relocate_links(text, source, destination):
    """Rebase ordinary Markdown links while preserving their fragments."""
    def replace(match):
        label, target = match.groups()
        if re.match(r"[a-zA-Z][a-zA-Z0-9+.-]*:", target):
            return match[0]
        path, separator, fragment = target.partition("#")
        resolved = (source.parent / path).resolve() if path else source.resolve()
        relative = os.path.relpath(resolved, destination.parent).replace("\\", "/")
        return f"[{label}]({relative}{separator}{fragment})"

    return re.sub(r"\[([^\]\n]+)\]\(([^)\n]+)\)", replace, text)


def render(root, spec):
    source = root / spec["source"]
    destination = root / ".agents" / "skills" / spec["name"] / "SKILL.md"
    body = section(source.read_text(encoding="utf-8"), spec["heading"])
    body = relocate_links(body, source, destination)
    source_link = os.path.relpath(source, destination.parent).replace("\\", "/")
    anchor = spec["heading"].lower().replace(" ", "-")
    text = (
        f"---\nname: {spec['name']}\ndescription: {spec['description']}\n---\n\n"
        f"# {spec['title']}\n\n"
        f"> Generated from [{spec['source']}#{anchor}]({source_link}#{anchor}).\n"
        "> Edit that procedure, then run `python tools/compile_project_skills.py` from the repository root.\n"
        "> Check freshness with `python tools/compile_project_skills.py --check`. Do not edit this generated file.\n\n"
        f"## Goal\n\n{spec['goal']}\n\n## Steps\n\n{body}\n"
    )
    return destination, text


def compile_skills(root=ROOT, check=False):
    # Render all inputs first; a missing source must not produce a partial update.
    outputs = [render(root, spec) for spec in SPECS]
    stale = []
    for path, text in outputs:
        if check:
            if not path.exists() or path.read_text(encoding="utf-8") != text:
                stale.append(path.relative_to(root).as_posix())
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8", newline="\n")
    return stale


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Report stale or missing skills without writing files")
    args = parser.parse_args()
    try:
        stale = compile_skills(check=args.check)
    except (OSError, ValueError) as error:
        parser.exit(1, f"Cannot compile project skills: {error}\n")
    if stale:
        parser.exit(1, "Stale generated skills:\n" + "\n".join(stale) + "\n")
    print("Generated skills are current." if args.check else f"Generated {len(SPECS)} project skill(s).")


if __name__ == "__main__":
    main()
