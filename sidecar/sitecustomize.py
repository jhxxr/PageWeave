"""Auto-imported at Python startup (PyInstaller puts _internal/ on sys.path).

Patches the openai client so its HTTP User-Agent is not the default
`OpenAI/Python <ver>` because some OpenAI-compatible gateways (e.g. api.xinr.de)
filter on that string and return `Your request was blocked` for it. We
override to a neutral `PageWeave/<ver>` UA so BabelDOC's OpenAITranslator
works against such gateways without modifying BabelDOC's source.

The patch is defensive: if openai isn't importable or the shape changes,
we silently no-op so the rest of babeldoc still loads.
"""

try:  # pragma: no cover - startup guard
    import openai

    _PATCHED_UA = "PageWeave/0.1"

    def _patch(cls):
        _orig = cls.__init__

        def _wrapped(self, *args, **kwargs):
            dh = dict(kwargs.get("default_headers") or {})
            dh.setdefault("User-Agent", _PATCHED_UA)
            kwargs["default_headers"] = dh
            _orig(self, *args, **kwargs)

        cls.__init__ = _wrapped

    for _name in ("OpenAI", "AsyncOpenAI", "AzureOpenAI", "AsyncAzureOpenAI"):
        _cls = getattr(openai, _name, None)
        if _cls is not None and hasattr(_cls, "__init__"):
            _patch(_cls)
except Exception:
    pass
