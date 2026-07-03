"""Entry point for the bundled BabelDOC sidecar.

The standalone `babeldoc` console script does not survive PyInstaller cleanly in all
environments, so we dispatch to its real CLI entry ourselves. This thin wrapper just
calls `babeldoc.main:cli` with sys.argv, preserving the exact CLI semantics the Rust
runner expects (same args, same stderr progress output).
"""
import sys
import multiprocessing

def main() -> int:
    multiprocessing.freeze_support()
    try:
        from babeldoc.main import cli
    except Exception as e:  # pragma: no cover - startup guard
        sys.stderr.write(f"[babeldoc-sidecar] failed to import babeldoc.main: {e}\n")
        return 1
    try:
        cli()
        return 0
    except SystemExit as e:
        # click/argparse raise SystemExit; honor its code.
        code = e.code
        if isinstance(code, int):
            return code
        return 0 if code is None else 1
    except Exception as e:  # pragma: no cover
        sys.stderr.write(f"[babeldoc-sidecar] error: {e}\n")
        return 1

if __name__ == "__main__":
    sys.exit(main())
