from pathlib import Path

import openpyxl

from parsers.xlsx_parser import parse_xlsx


def _write_workbook(path: Path, rows: list[list[object]]) -> None:
    wb = openpyxl.Workbook()
    ws = wb.active
    for row in rows:
        ws.append(row)
    wb.save(path)


def test_parses_header_and_rows(tmp_path: Path) -> None:
    xlsx_file = tmp_path / "products.xlsx"
    _write_workbook(
        xlsx_file,
        [
            ["name", "price", "in_stock"],
            ["Facial Cleanser", 18.5, "yes"],
            ["Vitamin C Serum", 32.0, "no"],
        ],
    )

    rows = parse_xlsx(xlsx_file)

    assert rows == [
        {"name": "Facial Cleanser", "price": 18.5, "in_stock": "yes"},
        {"name": "Vitamin C Serum", "price": 32.0, "in_stock": "no"},
    ]


def test_keeps_native_numeric_types_unlike_csv(tmp_path: Path) -> None:
    # This is the deliberate difference from parse_csv: xlsx has real
    # types, so a numeric cell stays a float/int, not a string.
    xlsx_file = tmp_path / "quantities.xlsx"
    _write_workbook(xlsx_file, [["item", "qty"], ["Bottle", 3]])

    rows = parse_xlsx(xlsx_file)

    assert rows[0]["qty"] == 3
    assert isinstance(rows[0]["qty"], int)


def test_header_only_returns_no_rows(tmp_path: Path) -> None:
    xlsx_file = tmp_path / "empty.xlsx"
    _write_workbook(xlsx_file, [["name", "price"]])

    assert parse_xlsx(xlsx_file) == []
