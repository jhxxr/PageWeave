"""Entry point for the bundled markitdown sidecar.

Thin wrapper around markitdown's CLI so the Rust convert runner can spawn a
stable exe with the same argv semantics as `markitdown <file> -o out.md`.
"""
import multiprocessing
import sys


def main() -> int:
    multiprocessing.freeze_support()
    try:
        from markitdown.__main__ import main as markitdown_main
    except Exception as e:  # pragma: no cover - startup guard
        sys.stderr.write(f"[markitdown-sidecar] failed to import markitdown: {e}\n")
        return 1
    try:
        result = markitdown_main()
        if isinstance(result, int):
            return result
        return 0
    except SystemExit as e:
        code = e.code
        if isinstance(code, int):
            return code
        return 0 if code is None else 1
    except Exception as e:  # pragma: no cover
        sys.stderr.write(f"[markitdown-sidecar] error: {e}\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
