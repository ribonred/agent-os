import csv
from pathlib import Path


def parse_csv(path: Path) -> list[dict[str, str]]:
    """Parse a CSV file into a list of row dicts keyed by header column.

    Deliberately minimal: no type coercion, no schema validation here --
    every value stays a string. Turning "3" into an int, or deciding a
    column means something specific, belongs to the extraction step, not
    parsing. Parsing's only job is getting the file's literal content out
    without guessing at meaning.
    """
    with path.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        return list(reader)
