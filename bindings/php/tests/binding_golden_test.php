<?php

declare(strict_types=1);

if (!extension_loaded('celestial-php')) {
    fwrite(STDERR, "celestial-php extension is not loaded\n");
    exit(1);
}

/** @return array<mixed> */
function expectArray(mixed $value): array {
    if (!is_array($value)) {
        throw new RuntimeException('expected array in binding golden fixture');
    }
    return $value;
}

function assertClose(float $actual, float $expected, float $tolerance, string $label): void {
    if (!is_finite($actual) || abs($actual - $expected) > $tolerance) {
        throw new RuntimeException("$label: expected $expected, got $actual");
    }
}

/**
 * @param array<mixed> $actual
 * @param array<mixed> $expected
 */
function assertVectorClose(
    array $actual,
    array $expected,
    float $tolerance,
    string $label
): void {
    if (count($actual) !== count($expected)) {
        throw new RuntimeException("$label: vector length mismatch");
    }
    foreach ($actual as $index => $value) {
        assertClose((float)$value, (float)$expected[$index], $tolerance, "{$label}[$index]");
    }
}

$fixture_path = __DIR__ . '/../../../tests/fixtures/binding_golden.json';
$fixture_json = file_get_contents($fixture_path);
if ($fixture_json === false) {
    throw new RuntimeException("cannot read $fixture_path");
}
$fixture = expectArray(json_decode($fixture_json, true, flags: JSON_THROW_ON_ERROR));
$tolerance = (float)$fixture['tolerance'];

$calc = expectArray($fixture['calc_ut']);
foreach (expectArray($calc['cases']) as $raw_case) {
    $case = expectArray($raw_case);
    $position = calc_ut((float)$calc['jd'], (int)$case['body'], (int)$calc['flags']);
    assertVectorClose(
        $position,
        expectArray($case['position']),
        $tolerance,
        "calc_ut body {$case['body']}"
    );
}

$houses = expectArray($fixture['houses_ex']);
$house_result = houses_ex(
    (float)$houses['jd'],
    (float)$houses['lat'],
    (float)$houses['lon'],
    (int)$houses['hsys'],
    (int)$houses['flags']
);
assertVectorClose(
    expectArray($house_result['cusps']),
    expectArray($houses['cusps']),
    $tolerance,
    'houses_ex cusps'
);
assertVectorClose(
    expectArray($house_result['ascmc']),
    expectArray($houses['ascmc']),
    $tolerance,
    'houses_ex ascmc'
);

$rise = expectArray($fixture['rise_trans']);
$rise_result = rise_trans(
    (float)$rise['jd'],
    (int)$rise['planet'],
    (int)$rise['flags'],
    (int)$rise['event_type'],
    expectArray($rise['geopos']),
    (float)$rise['pressure_mb'],
    (float)$rise['temp_c']
);
assertClose(
    (float)$rise_result['ret'][0],
    (float)$rise['ret_flags'],
    0.0,
    'rise_trans ret_flags'
);
assertClose(
    (float)$rise_result['tret'][0],
    (float)$rise['tret'],
    $tolerance,
    'rise_trans tret'
);

echo "binding golden parity ok\n";
