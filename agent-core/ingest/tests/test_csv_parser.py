from pathlib import Path

from parsers.csv_parser import parse_csv


def test_parses_header_and_rows(tmp_path: Path) -> None:
    csv_file = tmp_path / "products.csv"
    csv_file.write_text(
        "name,price,in_stock\n"
        "Facial Cleanser,18.50,yes\n"
        "Vitamin C Serum,32.00,no\n",
        encoding="utf-8",
    )

    rows = parse_csv(csv_file)

    assert rows == [
        {"name": "Facial Cleanser", "price": "18.50", "in_stock": "yes"},
        {"name": "Vitamin C Serum", "price": "32.00", "in_stock": "no"},
    ]


def test_empty_file_with_only_header_returns_no_rows(tmp_path: Path) -> None:
    csv_file = tmp_path / "empty.csv"
    csv_file.write_text("name,price\n", encoding="utf-8")

    assert parse_csv(csv_file) == []


def test_values_stay_strings_not_coerced(tmp_path: Path) -> None:
    # Parsing must not guess types -- "3" stays "3", not 3. That decision
    # belongs to the extraction step, not here.
    csv_file = tmp_path / "quantities.csv"
    csv_file.write_text("item,qty\nBottle,3\n", encoding="utf-8")

    rows = parse_csv(csv_file)

    assert rows[0]["qty"] == "3"
    assert isinstance(rows[0]["qty"], str)
