#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    use crate::correlation::*;
    use crate::structs::*;

    /// Helper to create a Decimal from string with 7+ decimal places
    fn decimal(val: &str) -> Decimal {
        Decimal::from_str(val).expect("Failed to parse decimal")
    }

    // ============================================================================
    // TEST 1: Basic Precision - Decimal coordinates maintain 7+ decimals
    // ============================================================================
    #[test]
    fn test_decimal_precision_preserved() {
        let point = AdressClean {
            coordinates: [decimal("13.1881234567890"), decimal("55.6048765432109")],
            postnummer: "200 00".to_string(),
            adress: "Test Address".to_string(),
            gata: "Test Street".to_string(),
            gatunummer: "1".to_string(),
        };

        // Verify we have at least 7 decimal places
        let coord_x_str = point.coordinates[0].to_string();
        let coord_y_str = point.coordinates[1].to_string();

        // Count decimal places
        let x_decimals = coord_x_str
            .split('.')
            .nth(1)
            .map(|s| s.len())
            .unwrap_or(0);
        let y_decimals = coord_y_str
            .split('.')
            .nth(1)
            .map(|s| s.len())
            .unwrap_or(0);

        assert!(
            x_decimals >= 7,
            "X coordinate should have at least 7 decimals, got {}",
            x_decimals
        );
        assert!(
            y_decimals >= 7,
            "Y coordinate should have at least 7 decimals, got {}",
            y_decimals
        );
    }

    // ============================================================================
    // TEST 2: Exact Match - Address directly on parking line
    // ============================================================================
    #[test]
    fn test_exact_match_distance_zero() {
        let point = AdressClean {
            coordinates: [decimal("13.1881234"), decimal("55.6048765")],
            postnummer: "200 00".to_string(),
            adress: "Main Street 1".to_string(),
            gata: "Main Street".to_string(),
            gatunummer: "1".to_string(),
        };

        // Parking line with both points identical to address
        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.1881234"), decimal("55.6048765")],
                [decimal("13.1881234"), decimal("55.6048765")],
            ],
            info: "Miljö Parking Zone A".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = find_closest_lines(&points, &lines);
        assert_eq!(results.len(), 1, "Should return 1 result");

        let result = &results[0];
        assert!(result.is_some(), "Result should not be None");

        let (index, distance) = result.unwrap();
        assert_eq!(index, 0, "Should match the first line");
        assert_eq!(distance, decimal("0"), "Distance should be 0 for exact match");
    }

    // ============================================================================
    // TEST 3: Within Threshold - Close but valid match (dist < 0.001)
    // ============================================================================
    #[test]
    fn test_within_threshold() {
        // Malmö coordinates with real-world precision
        let point = AdressClean {
            coordinates: [decimal("13.188123"), decimal("55.604876")],
            postnummer: "200 00".to_string(),
            adress: "Storgatan 15".to_string(),
            gata: "Storgatan".to_string(),
            gatunummer: "15".to_string(),
        };

        // Parking zone endpoint very close (within 0.001)
        // ~0.00008 distance (roughly 8 meters at equator)
        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.188000"), decimal("55.604800")],
                [decimal("13.188300"), decimal("55.605000")],
            ],
            info: "Miljö Parking Zone B".to_string(),
            tid: "09:00-17:00".to_string(),
            dag: 2,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = find_closest_lines(&points, &lines);
        let (_, distance) = results[0].unwrap();

        let threshold = decimal("0.001");
        assert!(
            distance < threshold,
            "Distance {} should be less than threshold {}",
            distance,
            threshold
        );
    }

    // ============================================================================
    // TEST 4: Rejection Test - Address outside threshold (dist > 0.001)
    // ============================================================================
    #[test]
    fn test_outside_threshold() {
        let point = AdressClean {
            coordinates: [decimal("13.200000"), decimal("55.600000")],
            postnummer: "200 00".to_string(),
            adress: "Far Away Street 1".to_string(),
            gata: "Far Away Street".to_string(),
            gatunummer: "1".to_string(),
        };

        // Parking zone far away (distance > 0.001)
        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.100000"), decimal("55.500000")],
                [decimal("13.100100"), decimal("55.500100")],
            ],
            info: "Distant Parking Zone".to_string(),
            tid: "10:00-16:00".to_string(),
            dag: 3,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = find_closest_lines(&points, &lines);
        let (_, distance) = results[0].unwrap();

        let threshold = decimal("0.001");
        assert!(
            distance > threshold,
            "Distance {} should be greater than threshold {}",
            distance,
            threshold
        );
    }

    // ============================================================================
    // TEST 5: Correct Assignment - Multiple lines, should pick closest
    // ============================================================================
    #[test]
    fn test_multiple_lines_closest_selected() {
        let point = AdressClean {
            coordinates: [decimal("13.1881234"), decimal("55.6048765")],
            postnummer: "200 00".to_string(),
            adress: "Test Address".to_string(),
            gata: "Test Street".to_string(),
            gatunummer: "1".to_string(),
        };

        // Three parking zones
        let line1 = MiljoeDataClean {
            coordinates: [
                [decimal("13.1000000"), decimal("55.6000000")], // FAR
                [decimal("13.1050000"), decimal("55.6050000")],
            ],
            info: "Far Parking Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let line2 = MiljoeDataClean {
            coordinates: [
                [decimal("13.1880000"), decimal("55.6040000")], // CLOSEST
                [decimal("13.1890000"), decimal("55.6050000")],
            ],
            info: "Closest Parking Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let line3 = MiljoeDataClean {
            coordinates: [
                [decimal("13.2000000"), decimal("55.6200000")], // FAR
                [decimal("13.2050000"), decimal("55.6250000")],
            ],
            info: "Other Far Parking Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line1, line2, line3];

        let results = find_closest_lines(&points, &lines);
        let (index, _) = results[0].unwrap();

        assert_eq!(
            index, 1,
            "Should select line index 1 (the closest zone), got {}",
            index
        );
    }

    // ============================================================================
    // TEST 6: Correlation Output - Correct AdressInfo generation
    // ============================================================================
    #[test]
    fn test_correlation_output_structure() {
        let point = AdressClean {
            coordinates: [decimal("13.1881234"), decimal("55.6048765")],
            postnummer: "202 00".to_string(),
            adress: "Storgatan 15".to_string(),
            gata: "Storgatan".to_string(),
            gatunummer: "15".to_string(),
        };

        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.1880000"), decimal("55.6048000")],
                [decimal("13.1890000"), decimal("55.6050000")],
            ],
            info: "Parking Zone A".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = correlation(points, lines);
        assert_eq!(results.len(), 1, "Should have 1 correlation result");

        let result = &results[0];
        assert!(result.relevant, "Should be marked as relevant (within threshold)");
        assert_eq!(result.adress, "Storgatan 15", "Address should match");
        assert_eq!(result.gata, "Storgatan", "Street should match");
        assert_eq!(result.gatunummer, "15", "Street number should match");
        assert_eq!(
            result.info, "Parking Zone A",
            "Parking info should match the linked zone"
        );
        assert_eq!(result.tid, "08:00-18:00", "Time info should match");
        assert_eq!(result.dag, 1, "Day should match");
    }

    // ============================================================================
    // TEST 7: Multiple Addresses - Batch processing
    // ============================================================================
    #[test]
    fn test_multiple_addresses_correlation() {
        let points = vec![
            AdressClean {
                coordinates: [decimal("13.1881234"), decimal("55.6048765")],
                postnummer: "200 00".to_string(),
                adress: "Storgatan 1".to_string(),
                gata: "Storgatan".to_string(),
                gatunummer: "1".to_string(),
            },
            AdressClean {
                coordinates: [decimal("13.1950000"), decimal("55.6100000")],
                postnummer: "201 00".to_string(),
                adress: "Lilla Torg 5".to_string(),
                gata: "Lilla Torg".to_string(),
                gatunummer: "5".to_string(),
            },
            AdressClean {
                coordinates: [decimal("13.5000000"), decimal("55.9000000")], // VERY FAR
                postnummer: "202 00".to_string(),
                adress: "Västra Varvsgatan 10".to_string(),
                gata: "Västra Varvsgatan".to_string(),
                gatunummer: "10".to_string(),
            },
        ];

        let lines = vec![
            MiljoeDataClean {
                coordinates: [
                    [decimal("13.1880000"), decimal("55.6040000")],
                    [decimal("13.1890000"), decimal("55.6050000")],
                ],
                info: "Zone A".to_string(),
                tid: "08:00-18:00".to_string(),
                dag: 1,
            },
            MiljoeDataClean {
                coordinates: [
                    [decimal("13.1950000"), decimal("55.6090000")],
                    [decimal("13.1960000"), decimal("55.6110000")],
                ],
                info: "Zone B".to_string(),
                tid: "09:00-17:00".to_string(),
                dag: 2,
            },
        ];

        let results = correlation(points, lines);
        assert_eq!(results.len(), 3, "Should have 3 correlation results");

        // Debug print
        for (i, result) in results.iter().enumerate() {
            eprintln!("Result {}: {} - relevant: {}", i, result.adress, result.relevant);
        }

        // First address should correlate with Zone A
        assert_eq!(results[0].adress, "Storgatan 1", "First result should be Storgatan 1");
        assert!(results[0].relevant, "Storgatan 1 should be relevant");

        // Second address should correlate with Zone B
        assert_eq!(results[1].adress, "Lilla Torg 5", "Second result should be Lilla Torg 5");
        assert!(results[1].relevant, "Lilla Torg 5 should be relevant");

        // Third address is very far (not relevant)
        assert_eq!(results[2].adress, "Västra Varvsgatan 10", "Third result should be Västra Varvsgatan 10");
        assert!(!results[2].relevant, "Västra Varvsgatan 10 should NOT be relevant");
    }

    // ============================================================================
    // TEST 8: Edge Case - Degenerate line segment (both endpoints identical)
    // ============================================================================
    #[test]
    fn test_degenerate_line_segment() {
        let point = AdressClean {
            coordinates: [decimal("13.1881234"), decimal("55.6048765")],
            postnummer: "200 00".to_string(),
            adress: "Test".to_string(),
            gata: "Test".to_string(),
            gatunummer: "1".to_string(),
        };

        // Degenerate segment (start == end)
        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.1880000"), decimal("55.6040000")],
                [decimal("13.1880000"), decimal("55.6040000")], // Same as start
            ],
            info: "Point Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = find_closest_lines(&points, &lines);
        assert!(results[0].is_some(), "Should handle degenerate segment");
        assert!(results[0].unwrap().1 > decimal("0"));
    }

    // ============================================================================
    // TEST 9: Threshold Calibration - Test with different thresholds
    // ============================================================================
    #[test]
    fn test_threshold_calibration_values() {
        let point = AdressClean {
            coordinates: [decimal("13.1881234"), decimal("55.6048765")],
            postnummer: "200 00".to_string(),
            adress: "Test".to_string(),
            gata: "Test".to_string(),
            gatunummer: "1".to_string(),
        };

        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.1870000"), decimal("55.6040000")],
                [decimal("13.1890000"), decimal("55.6050000")],
            ],
            info: "Parking Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = find_closest_lines(&points, &lines);
        let distance = results[0].unwrap().1;

        // Test against multiple threshold values
        // Distance is approximately 0.000281
        let test_thresholds = [
            ("0.00001", false),  // 1 meter - too small
            ("0.0001", false),   // 10 meters - too small
            ("0.001", true),     // 100 meters - should pass (distance 0.000281 < 0.001)
            ("0.01", true),      // 1000 meters - should pass
            ("0.1", true),       // 10km - definitely passes
        ];

        for (threshold_str, should_pass) in &test_thresholds {
            let threshold = decimal(threshold_str);
            let is_within = distance < threshold;
            assert_eq!(
                is_within, *should_pass,
                "Distance {} vs threshold {} failed (expected {})",
                distance, threshold, should_pass
            );
        }
    }

    // ============================================================================
    // TEST 10: Real-world Malmö Coordinates
    // ============================================================================
    #[test]
    fn test_real_world_malmo_coordinates() {
        // Real Malmö addresses
        let points = vec![
            AdressClean {
                coordinates: [decimal("13.1945945"), decimal("55.5932645")],
                postnummer: "211 00".to_string(),
                adress: "Lilla Torg 1".to_string(),
                gata: "Lilla Torg".to_string(),
                gatunummer: "1".to_string(),
            },
            AdressClean {
                coordinates: [decimal("13.2004523"), decimal("55.6043210")],
                postnummer: "213 00".to_string(),
                adress: "Västra Varvsgatan 41".to_string(),
                gata: "Västra Varvsgatan".to_string(),
                gatunummer: "41".to_string(),
            },
        ];

        // Real Malmö parking zones
        let lines = vec![
            MiljoeDataClean {
                coordinates: [
                    [decimal("13.1940000"), decimal("55.5930000")],
                    [decimal("13.1950000"), decimal("55.5935000")],
                ],
                info: "Lilla Torg Miljözon".to_string(),
                tid: "08:00-18:00".to_string(),
                dag: 1,
            },
            MiljoeDataClean {
                coordinates: [
                    [decimal("13.2000000"), decimal("55.6040000")],
                    [decimal("13.2010000"), decimal("55.6045000")],
                ],
                info: "Västra Varvsgatan Miljözon".to_string(),
                tid: "09:00-17:00".to_string(),
                dag: 2,
            },
        ];

        let results = correlation(points.clone(), lines);

        // Both should be relevant
        for result in &results {
            assert!(
                result.relevant,
                "Real-world Malmö address should be relevant: {}",
                result.adress
            );
        }

        // Verify correct associations
        assert_eq!(
            results[0].info, "Lilla Torg Miljözon",
            "Lilla Torg should match with correct zone"
        );
        assert_eq!(
            results[1].info, "Västra Varvsgatan Miljözon",
            "Västra Varvsgatan should match with correct zone"
        );
    }

    // ============================================================================
    // TEST 11: Precision Loss Detection - Ensure no rounding errors
    // ============================================================================
    #[test]
    fn test_no_precision_loss_in_calculations() {
        // Very close coordinates that would lose precision with f64
        let point = AdressClean {
            coordinates: [decimal("13.18812345678901"), decimal("55.60487654321098")],
            postnummer: "200 00".to_string(),
            adress: "Precision Test".to_string(),
            gata: "Precision".to_string(),
            gatunummer: "1".to_string(),
        };

        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.18812345678901"), decimal("55.60487654321098")],
                [decimal("13.18812345678902"), decimal("55.60487654321099")],
            ],
            info: "Precision Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = find_closest_lines(&points, &lines);
        let (_, distance) = results[0].unwrap();

        // Should be extremely small (near zero) for identical coordinate
        assert!(
            distance < decimal("0.0001"),
            "Precision test distance should be near zero, got {}",
            distance
        );
    }

    // ============================================================================
    // TEST 12: Performance - Batch with many addresses and zones
    // ============================================================================
    #[test]
    fn test_batch_performance_many_records() {
        // Generate 100 test addresses
        let mut points = Vec::new();
        for i in 0..100 {
            let lat_offset = Decimal::from(i) * decimal("0.0001");
            points.push(AdressClean {
                coordinates: [decimal("13.1881234") + lat_offset, decimal("55.6048765")],
                postnummer: "Cant be bothered to fix this shit yet".to_string(),
                adress: format!("Address {}", i),
                gata: "Test Street".to_string(),
                gatunummer: format!("{}", i),
            });
        }

        // Generate 50 parking zones
        let mut lines = Vec::new();
        for i in 0..50 {
            let lat_offset = Decimal::from(i) * decimal("0.0002");
            lines.push(MiljoeDataClean {
                coordinates: [
                    [decimal("13.1881000") + lat_offset, decimal("55.6048000")],
                    [decimal("13.1890000") + lat_offset, decimal("55.6050000")],
                ],
                info: format!("Zone {}", i),
                tid: "08:00-18:00".to_string(),
                dag: (1 + (i as u8) % 7),
            });
        }

        let results = correlation(points, lines);

        // Verify results
        assert_eq!(results.len(), 100, "Should return 100 results");
        assert!(
            results.iter().filter(|r| r.relevant).count() > 0,
            "At least some addresses should be relevant"
        );
    }
}
