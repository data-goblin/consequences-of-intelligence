import json
import re
import sys
from pathlib import Path
from typing import Any

MAX_WORDS = 60
MAX_SENTENCES = 3
MAX_LABEL_WORDS = 8
THEMES = {"ip", "mind", "work", "security", "power", "information", "world"}
BANNED = {"—": "em-dash", "·": "middle dot"}
MARKER = re.compile(r"\[(\d+)\]")
SENTENCE_END = re.compile(r"[.!?][\"')\]]*(?:\[\d+\])*(?=\s+[A-Z\"(]|\s*$)")


def words(text: str) -> int:
    return len(MARKER.sub("", text).split())


def sentences(text: str) -> int:
    return max(1, len(SENTENCE_END.findall(text.strip())))


def lint(data: dict[str, Any], raw: str) -> list[str]:
    problems: list[str] = []
    for char, name in BANNED.items():
        if char in raw:
            problems.append(f"data.json contains a {name}")
    nodes: list[dict[str, Any]] = data["nodes"]
    ids = [n["id"] for n in nodes]
    known = set(ids)
    for dup in sorted({i for i in ids if ids.count(i) > 1}):
        problems.append(f"{dup}: duplicate id")
    for n in nodes:
        nid = n["id"]
        if n.get("theme") not in THEMES:
            problems.append(f"{nid}: unknown theme {n.get('theme')!r}")
        if n.get("merged") and n["merged"] not in known:
            problems.append(f"{nid}: merged into unknown id {n['merged']}")
        refs = n.get("refs", [])
        for ref in refs:
            for field in ("title", "url", "finding"):
                if not ref.get(field):
                    problems.append(f"{nid}: a ref has no {field}")
        summary = n.get("summary", "")
        for marker in MARKER.findall(summary):
            if not 1 <= int(marker) <= len(refs):
                problems.append(f"{nid}: [{marker}] has no ref")
        if n.get("merged") or n.get("kind"):
            continue
        if words(summary) > MAX_WORDS:
            problems.append(f"{nid}: summary has {words(summary)} words, max {MAX_WORDS}")
        if sentences(summary) > MAX_SENTENCES:
            problems.append(f"{nid}: summary has {sentences(summary)} sentences, max {MAX_SENTENCES}")
        if len(n.get("label", "").split()) > MAX_LABEL_WORDS:
            problems.append(f"{nid}: label has more than {MAX_LABEL_WORDS} words")
    for link in data["links"]:
        for end in ("source", "target"):
            if link[end] not in known:
                problems.append(f"link {link['source']} -> {link['target']}: unknown {end}")
    return problems


def main() -> int:
    path = Path(sys.argv[1] if len(sys.argv) > 1 else Path(__file__).with_name("data.json"))
    raw = path.read_text(encoding="utf-8")
    problems = lint(json.loads(raw), raw)
    for p in problems:
        print(p)
    print(f"{len(problems)} problem{'s' if len(problems) != 1 else ''}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
