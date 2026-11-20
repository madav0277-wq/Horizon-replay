#!/usr/bin/env python3
"""Package the Horizon Replay native source bundle as an .hnmod archive."""
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "dist" / "horizon-replay.hnmod"
FILES = (
    Path("horizon-native.json"),
    Path("mixins/horizon-replay/horizon-mixin.toml"),
    Path("mixins/horizon-replay/api/Cargo.toml"),
    Path("mixins/horizon-replay/api/src/lib.rs"),
)

def main() -> None:
    missing = [str(path) for path in FILES if not (ROOT / path).is_file()]
    if missing:
        raise SystemExit("Missing bundle files: " + ", ".join(missing))

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(OUTPUT, "w", compression=ZIP_DEFLATED, compresslevel=9) as archive:
        for path in FILES:
            archive.write(ROOT / path, path.as_posix())
    print(f"Created {OUTPUT}")

if __name__ == "__main__":
    main()
