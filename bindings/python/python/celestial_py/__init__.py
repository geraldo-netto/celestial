"""
celestial_py — Python bindings for the Swiss Ephemeris.

The compiled extension module ``celestial_py._celestial_py`` is built with
``maturin develop`` (development) or ``maturin build`` (wheel).

Once compiled, you can write::

    import celestial_py as celestial
    jd = celestial.julday(2002, 1, 1, 0)
"""

try:
    from celestial_py._celestial_py import *  # noqa: F401, F403

    _EXTENSION_LOADED = True
except ImportError:
    # Extension not yet compiled. Tests that need it are guarded by
    # pytestmark / requires_extension / HAS_MODULE checks in the test files.
    _EXTENSION_LOADED = False
