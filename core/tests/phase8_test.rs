//! Tests extracted from new_features_test.rs — do not edit manually.

#[cfg(test)]
mod phase8_indigenous {
    use celestial_core::*;

    #[test]
    fn medicine_wheel_covers_all_360() {
        // Every degree should return a valid totem
        let animals = [
            "Snow Goose",
            "Otter",
            "Cougar",
            "Red Hawk",
            "Beaver",
            "Deer",
            "Flicker",
            "Sturgeon",
            "Brown Bear",
            "Raven",
            "Snake",
            "Elk",
        ];
        for i in 0..360u32 {
            let lon = i as f64 + 0.5;
            let (animal, _, _, _) = medicine_wheel_totem(lon);
            assert!(
                animals.contains(&animal),
                "lon {lon}: animal '{animal}' not in zodiac"
            );
        }
    }

    #[test]
    fn medicine_wheel_returns_four_elements() {
        let elements = ["Fire", "Earth", "Air", "Water"];
        for i in 0..360u32 {
            let (_, element, _, _) = medicine_wheel_totem(i as f64 + 0.5);
            assert!(elements.contains(&element), "element '{element}' not valid");
        }
    }

    #[test]
    fn egyptian_decan_covers_all_36() {
        // 36 decans of 10° each
        let mut seen = std::collections::HashSet::new();
        for i in 0..36u32 {
            let lon = i as f64 * 10.0 + 5.0;
            let (idx, _, _) = egyptian_decan(lon);
            seen.insert(idx);
        }
        assert_eq!(seen.len(), 36, "should see all 36 decan indices");
    }

    #[test]
    fn egyptian_decan_idx_in_range() {
        for i in 0..360u32 {
            let (idx, name, star) = egyptian_decan(i as f64 + 0.5);
            assert!(idx < 36, "decan idx {idx} >= 36");
            assert!(!name.is_empty(), "decan name empty at {i}°");
            assert!(!star.is_empty(), "decan star empty at {i}°");
        }
    }
}
