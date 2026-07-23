<?php

declare(strict_types=1);

if (!extension_loaded('celestial-php')) {
    fwrite(STDERR, "celestial-php extension is not loaded\n");
    exit(1);
}

/** @return array<mixed> */
function expect_array(mixed $value): array {
    if (!is_array($value)) {
        throw new RuntimeException('expected array in binding golden fixture');
    }
    return $value;
}

function assert_close(float $actual, float $expected, float $tolerance, string $label): void {
    if (!is_finite($actual) || abs($actual - $expected) > $tolerance) {
        throw new RuntimeException("$label: expected $expected, got $actual");
    }
}

/**
 * @param array<mixed> $actual
 * @param array<mixed> $expected
 */
function assert_vector_close(
    array $actual,
    array $expected,
    float $tolerance,
    string $label
): void {
    if (count($actual) !== count($expected)) {
        throw new RuntimeException("$label: vector length mismatch");
    }
    foreach ($actual as $index => $value) {
        assert_close((float)$value, (float)$expected[$index], $tolerance, "{$label}[$index]");
    }
}

$fixture_path = __DIR__ . '/../../../tests/fixtures/binding_golden.json';
$fixture_json = file_get_contents($fixture_path);
if ($fixture_json === false) {
    throw new RuntimeException("cannot read $fixture_path");
}
$fixture = expect_array(json_decode($fixture_json, true, flags: JSON_THROW_ON_ERROR));
$tolerance = (float)$fixture['tolerance'];

$calc = expect_array($fixture['calc_ut']);
foreach (expect_array($calc['cases']) as $raw_case) {
    $case = expect_array($raw_case);
    $position = calc_ut((float)$calc['jd'], (int)$case['body'], (int)$calc['flags']);
    assert_vector_close(
        $position,
        expect_array($case['position']),
        $tolerance,
        "calc_ut body {$case['body']}"
    );
}

$houses = expect_array($fixture['houses_ex']);
$house_result = houses_ex(
    (float)$houses['jd'],
    (float)$houses['lat'],
    (float)$houses['lon'],
    (int)$houses['hsys'],
    (int)$houses['flags']
);
assert_vector_close(
    expect_array($house_result['cusps']),
    expect_array($houses['cusps']),
    $tolerance,
    'houses_ex cusps'
);
assert_vector_close(
    expect_array($house_result['ascmc']),
    expect_array($houses['ascmc']),
    $tolerance,
    'houses_ex ascmc'
);

$rise = expect_array($fixture['rise_trans']);
$rise_result = rise_trans(
    (float)$rise['jd'],
    (int)$rise['planet'],
    (int)$rise['flags'],
    (int)$rise['event_type'],
    expect_array($rise['geopos']),
    (float)$rise['pressure_mb'],
    (float)$rise['temp_c']
);
assert_close(
    (float)$rise_result['ret'][0],
    (float)$rise['ret_flags'],
    0.0,
    'rise_trans ret_flags'
);
assert_close(
    (float)$rise_result['tret'][0],
    (float)$rise['tret'],
    $tolerance,
    'rise_trans tret'
);

echo "binding golden parity ok\n";
