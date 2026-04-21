<?php
/**
 * Adversarial boundary / fuzz tests for the PHP binding.
 *
 * Mirrors the Rust fuzz suites and the Python TestFuzzingBoundary class.
 * Requires the compiled extension:
 *   php -d extension=target/release/libcelestial.so tests/fuzz_boundary.php
 */

declare(strict_types=1);

$passed = 0;
$failed = 0;

function ok(bool $cond, string $label): void {
    global $passed, $failed;
    if ($cond) {
        $passed++;
    } else {
        $failed++;
        echo "FAIL: $label\n";
    }
}

// ── norm_deg: large inputs always land in [0, 360) ────────────────────────────

foreach ([1e15, -1e15, 1e100, -1e100, 0.0, 360.0, -360.0, 720.5] as $v) {
    if (is_finite($v)) {
        $n = norm_deg($v);
        ok($n >= 0.0 && $n < 360.0, "norm_deg($v) = $n in [0,360)");
    }
}

// ── norm_cs: large centisecond inputs ─────────────────────────────────────────

$FULL = 360 * 360000;
foreach ([0, 1, -1, $FULL, $FULL + 1, -$FULL, PHP_INT_MAX >> 8, -(PHP_INT_MAX >> 8)] as $v) {
    $n = norm_cs((int) $v);
    ok($n >= 0 && $n < $FULL, "norm_cs($v) = $n in [0,$FULL)");
}

// ── diff_deg_signed: result always in (-180, +180] ────────────────────────────

$angles = [0.0, 1.0, 90.0, 179.9, 180.0, 180.1, 270.0, 359.0, 360.0, -1.0, -180.0];
foreach ($angles as $a) {
    foreach ($angles as $b) {
        $d = diff_deg_signed($a, $b);
        ok($d > -180.0 && $d <= 180.0, "diff_deg_signed($a,$b) = $d in (-180,180]");
    }
}

// ── julday / revjul: 500 deterministic round-trips ────────────────────────────

$seed = 42;
$rand = function () use (&$seed): float {
    $seed = ($seed * 1664525 + 1013904223) & 0xFFFFFFFF;
    return ($seed & 0xFFFFFFFF) / 0x100000000;
};

for ($i = 0; $i < 500; $i++) {
    $y = (int) ($rand() * 500) + 1583;
    $m = (int) ($rand() * 12)  + 1;
    $d = (int) ($rand() * 28)  + 1;
    $h = $rand() * 24.0;
    $jd   = julday($y, $m, $d, $h, 1);
    $back = revjul($jd, 1);
    ok((int)$back['year']  === $y, "round-trip year  $y-$m-$d");
    ok((int)$back['month'] === $m, "round-trip month $y-$m-$d");
    ok((int)$back['day']   === $d, "round-trip day   $y-$m-$d");
}

// ── coord_transform: outputs always finite ────────────────────────────────────

foreach (range(0, 330, 30) as $lon) {
    foreach (range(-80, 80, 20) as $lat) {
        foreach ([-90.0, -23.44, 0.0, 23.44, 90.0] as $eps) {
            $out = coord_transform([(float)$lon, (float)$lat, 1.0], $eps);
            ok(is_finite($out[0]) && is_finite($out[1]),
               "coord_transform($lon,$lat,$eps) finite");
        }
    }
}

// ── Summary ───────────────────────────────────────────────────────────────────

echo "Results: $passed passed, $failed failed\n";
if ($failed > 0) {
    exit(1);
}
