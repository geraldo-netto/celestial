import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, test } from "vitest";
import type * as CelestialModule from "../index";

interface CalcCase {
  body: number;
  position: number[];
}

interface Golden {
  tolerance: number;
  calc_ut: {
    jd: number;
    flags: number;
    cases: CalcCase[];
  };
  houses_ex: {
    jd: number;
    lat: number;
    lon: number;
    hsys: number;
    flags: number;
    cusps: number[];
    ascmc: number[];
  };
  rise_trans: {
    jd: number;
    planet: number;
    flags: number;
    event_type: number;
    geopos: number[];
    pressure_mb: number;
    temp_c: number;
    ret_flags: number;
    tret: number;
  };
}

const celestial = require("../index") as typeof CelestialModule;
const golden = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/binding_golden.json"), "utf8"),
) as Golden;

function expectVectorClose(actual: number[], expected: number[]): void {
  expect(actual).toHaveLength(expected.length);
  actual.forEach((value, index) => {
    expect(Math.abs(value - expected[index])).toBeLessThanOrEqual(golden.tolerance);
  });
}

describe("native binding golden parity", () => {
  test("calcUt", () => {
    for (const fixture of golden.calc_ut.cases) {
      const pos = celestial.calcUt(golden.calc_ut.jd, fixture.body, golden.calc_ut.flags);
      expectVectorClose(
        [pos.lon, pos.lat, pos.dist, pos.speedLon, pos.speedLat, pos.speedDist],
        fixture.position,
      );
    }
  });

  test("housesEx", () => {
    const fixture = golden.houses_ex;
    const result = celestial.housesEx(
      fixture.jd,
      fixture.lat,
      fixture.lon,
      fixture.hsys,
      fixture.flags,
    );
    expectVectorClose(result.cusps, fixture.cusps);
    expectVectorClose(result.ascmc, fixture.ascmc);
  });

  test("riseTrans", () => {
    const fixture = golden.rise_trans;
    const result = celestial.riseTrans(
      fixture.jd,
      fixture.planet,
      fixture.flags,
      fixture.event_type,
      fixture.geopos,
      fixture.pressure_mb,
      fixture.temp_c,
    );
    expect(result.retFlags).toBe(fixture.ret_flags);
    expect(Math.abs(result.tret - fixture.tret)).toBeLessThanOrEqual(golden.tolerance);
  });
});
