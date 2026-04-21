"""
pytest configuration for celestial_py tests.

Provides shared fixtures and marks for tests that require:
  - the compiled Rust extension (``requires_extension``)
  - Swiss Ephemeris data files (``requires_ephe``)
"""

import os

import pytest


def pytest_configure(config: pytest.Config) -> None:
    """Register custom markers."""
    config.addinivalue_line(
        "markers",
        "requires_extension: test needs the compiled celestial_py extension",
    )
    config.addinivalue_line(
        "markers",
        "requires_ephe: test needs Swiss Ephemeris data files "
        "(set SWISSEPH_EPHE_PATH)",
    )


@pytest.fixture(scope="session")
def ephe_path() -> str:
    """Return the ephemeris path from the environment, or empty string."""
    return os.environ.get("SWISSEPH_EPHE_PATH", "")


@pytest.fixture(scope="session")
def has_extension() -> bool:
    """Return True if the compiled extension module is importable."""
    try:
        import celestial_py._celestial_py  # noqa: F401

        return True
    except ImportError:
        return False


@pytest.fixture(autouse=True)
def skip_without_extension(request: pytest.FixtureRequest, has_extension: bool) -> None:
    """Auto-skip tests marked ``requires_extension`` when the addon is absent."""
    if request.node.get_closest_marker("requires_extension") and not has_extension:
        pytest.skip("celestial_py extension not compiled — run `maturin develop`")


@pytest.fixture(autouse=True)
def skip_without_ephe(request: pytest.FixtureRequest, ephe_path: str) -> None:
    """Auto-skip tests marked ``requires_ephe`` when SWISSEPH_EPHE_PATH is unset."""
    if request.node.get_closest_marker("requires_ephe") and not ephe_path:
        pytest.skip("Set SWISSEPH_EPHE_PATH to run ephemeris tests")
