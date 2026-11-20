#!/usr/bin/env python3
"""Package and validate the Horizon Replay native source bundle as an .hnmod."""
import json
from pathlib import Path
import tomllib
from zipfile import ZIP_DEFLATED, ZipFile

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "dist" / "horizon-replay.hnmod"
FILES = (
    Path("horizon-native.json"),
    Path("mixins/horizon-replay/horizon-mixin.toml"),
    Path("mixins/horizon-replay/api/Cargo.toml"),
    Path("mixins/horizon-replay/api/src/lib.rs"),
)
EXPECTED_ID = "horizon-replay"


def main() -> None:
    missing = [str(path) for path in FILES if not (ROOT / path).is_file()]
    if missing:
        raise SystemExit("Missing bundle files: " + ", ".join(missing))

    metadata = json.loads((ROOT / FILES[0]).read_text(encoding="utf-8"))
    mixin = tomllib.loads((ROOT / FILES[1]).read_text(encoding="utf-8"))
    mod = mixin.get("mod", {})
    compatibility = mixin.get("compatibility", {})
    required = {"schema", "id", "version", "game", "horizon", "mixins"}
    if not required.issubset(metadata):
        raise SystemExit("horizon-native.json is missing: " + ", ".join(sorted(required - metadata.keys())))
    if set(metadata) - (required | {"hooks"}):
        raise SystemExit("horizon-native.json has unsupported fields")
    if metadata["schema"] != 1 or metadata["id"] != EXPECTED_ID:
        raise SystemExit("Unsupported schema or incorrect native mod id")
    if metadata["id"] != mod.get("id") or metadata["version"] != mod.get("version"):
        raise SystemExit("Native JSON and horizon-mixin.toml id/version do not match")
    if metadata["horizon"] != compatibility.get("horizon_revision"):
        raise SystemExit("Native JSON and horizon-mixin.toml Horizon revisions do not match")
    for field in ("horizon", "mixins"):
        value = metadata[field]
        if not isinstance(value, str) or len(value) != 40 or any(c not in "0123456789abcdef" for c in value):
            raise SystemExit(f"{field} must be a 40-character lowercase commit hash")

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(OUTPUT, "w", compression=ZIP_DEFLATED, compresslevel=9) as archive:
        for path in FILES:
            archive.write(ROOT / path, path.as_posix())

    # Re-open the archive and assert that the launcher-required source files are present.
    with ZipFile(OUTPUT) as archive:
        names = set(archive.namelist())
        expected = {path.as_posix() for path in FILES}
        if names != expected:
            raise SystemExit(f"Archive contents mismatch: {sorted(names ^ expected)}")
        if archive.testzip() is not None:
            raise SystemExit("Generated archive failed its ZIP integrity check")
    print(f"Created and validated {OUTPUT}")


if __name__ == "__main__":
    main()
