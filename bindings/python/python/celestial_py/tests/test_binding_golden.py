import json
import math
from pathlib import Path
from typing import Iterable

import celestial_py as celestial


def _fixture_path() -> Path:
    return Path(__file__).parents[5] / "tests" / "fixtures" / "binding_golden.json"


def _assert_vector_close(
    actual: Iterable[float], expected: Iterable[float], tolerance: float
) -> None:
    actual_values = list(actual)
    expected_values = list(expected)
    assert len(actual_values) == len(expected_values)
    for actual_value, expected_value in zip(actual_values, expected_values):
        assert math.isclose(
            actual_value, expected_value, rel_tol=0.0, abs_tol=tolerance
        )


def test_native_binding_golden_parity() -> None:
    assert celestial._EXTENSION_LOADED, "compiled celestial_py extension is required"
    fixture = json.loads(_fixture_path().read_text(encoding="utf-8"))
    tolerance = fixture["tolerance"]

    calc_fixture = fixture["calc_ut"]
    for case in calc_fixture["cases"]:
        position, _ret_flags = celestial.calc_ut(
            calc_fixture["jd"], case["body"], calc_fixture["flags"]
        )
        _assert_vector_close(position, case["position"], tolerance)

    houses_fixture = fixture["houses_ex"]
    cusps, ascmc = celestial.houses_ex(
        houses_fixture["jd"],
        houses_fixture["lat"],
        houses_fixture["lon"],
        houses_fixture["hsys"],
        houses_fixture["flags"],
    )
    _assert_vector_close(cusps, houses_fixture["cusps"], tolerance)
    _assert_vector_close(ascmc, houses_fixture["ascmc"], tolerance)

    rise_fixture = fixture["rise_trans"]
    ret_flags, tret = celestial.rise_trans(
        rise_fixture["jd"],
        rise_fixture["planet"],
        rise_fixture["flags"],
        rise_fixture["event_type"],
        rise_fixture["geopos"],
        rise_fixture["pressure_mb"],
        rise_fixture["temp_c"],
    )
    assert ret_flags == rise_fixture["ret_flags"]
    assert math.isclose(tret, rise_fixture["tret"], rel_tol=0.0, abs_tol=tolerance)
