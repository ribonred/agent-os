from pathlib import Path
from typing import Any

import openpyxl


def parse_xlsx(path: Path) -> list[dict[str, Any]]:
    """Parse the first worksheet of an xlsx file into a list of row dicts
    keyed by the header row.

    Unlike parse_csv, values keep openpyxl's native types (str, int,
    float, None) instead of being forced to strings -- xlsx actually has
    types and CSV doesn't, so flattening that away here would throw out
    real information the extraction step might care about.
    """
    workbook = openpyxl.load_workbook(path, read_only=True, data_only=True)
    sheet = workbook.active

    rows_iter = sheet.iter_rows(values_only=True)
    header = next(rows_iter, None)
    if header is None:
        return []

    return [dict(zip(header, row)) for row in rows_iter]
