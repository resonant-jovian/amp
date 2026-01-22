#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    use crate::correlation::*;
    use crate::structs::*;

    /// Helper to create a Decimal from string
    fn decimal(val: &str) -> Decimal {
        Decimal::from_str(val).expect("Failed to parse decimal")
    }

    // ============================================================================
    // TEST 1: Basic Precision - Decimal coordinates maintain high precision
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
    // TEST 2: Coordinate System Conversion - SWEREF99 TM to WGS84
    // ============================================================================
    #[test]
    fn test_sweref_coordinate_conversion() {
        // SWEREF99 TM coordinates for a known Malmö location
        // These should convert successfully to valid lat/lon
        let point = AdressClean {
            coordinates: [decimal("389000"), decimal("6164000")], // Malmö area
            postnummer: "200 00".to_string(),
            adress: "Test Address".to_string(),
            gata: "Test Street".to_string(),
            gatunummer: "1".to_string(),
        };

        // Parking zone with similar coordinates
        let line = MiljoeDataClean {
            coordinates: [
                [decimal("389000"), decimal("6164000")],
                [decimal("389100"), decimal("6164100")],
            ],
            info: "Test Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        // Should not panic during conversion
        let results = find_closest_lines(&points, &lines);
        assert_eq!(results.len(), 1, "Should return 1 result");
        assert!(results[0].is_some(), "Result should not be None");
    }

    // ============================================================================
    // TEST 3: Haversine Distance - Within 50m threshold (RELEVANT)
    // ============================================================================
    #[test]
    fn test_within_50m_threshold_relevant() {
        // Real Malmö coordinates: Storgatan area
        // These coordinates are ~30 meters apart
        let point = AdressClean {
            coordinates: [decimal("13.195000"), decimal("55.595000")],
            postnummer: "200 00".to_string(),
            adress: "Storgatan 1".to_string(),
            gata: "Storgatan".to_string(),
            gatunummer: "1".to_string(),
        };

        // Parking zone ~30 meters away
        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.194700"), decimal("55.594800")], // ~30m away
                [decimal("13.195300"), decimal("55.595200")], // ~30m away
            ],
            info: "Malmö Miljözon A".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = correlation(points, lines);
        assert_eq!(results.len(), 1, "Should return 1 result");

        let result = &results[0];
        // Distance should be ~30 meters (well within 50m threshold)
        assert!(
            result.relevant,
            "Address within 50m should be marked relevant"
        );
        assert_eq!(result.info, "Malmö Miljözon A");
        assert_eq!(result.tid, "08:00-18:00");
    }

    // ============================================================================
    // TEST 4: Distance Rejection - Beyond 50m threshold (NOT RELEVANT)
    // ============================================================================
    #[test]
    fn test_beyond_50m_threshold_not_relevant() {
        // Malmö address
        let point = AdressClean {
            coordinates: [decimal("13.195000"), decimal("55.595000")],
            postnummer: "200 00".to_string(),
            adress: "Storgatan 1".to_string(),
            gata: "Storgatan".to_string(),
            gatunummer: "1".to_string(),
        };

        // Parking zone ~200 meters away (far beyond 50m threshold)
        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.193000"), decimal("55.593000")], // ~200m away
                [decimal("13.193200"), decimal("55.593200")],
            ],
            info: "Distant Zone".to_string(),
            tid: "09:00-17:00".to_string(),
            dag: 2,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = correlation(points, lines);
        assert_eq!(results.len(), 1);

        let result = &results[0];
        // Should be marked as NOT relevant (too far)
        assert!(
            !result.relevant,
            "Address beyond 50m should NOT be marked relevant"
        );
        // Even when not relevant, should still have address info
        assert_eq!(result.adress, "Storgatan 1");
    }

    // ============================================================================
    // TEST 5: Exact Location Match - Zero distance
    // ============================================================================
    #[test]
    fn test_exact_location_match() {
        let coord = [decimal("13.195000"), decimal("55.595000")];

        let point = AdressClean {
            coordinates: coord,
            postnummer: "200 00".to_string(),
            adress: "Test Address".to_string(),
            gata: "Test Street".to_string(),
            gatunummer: "1".to_string(),
        };

        // Parking zone at identical location
        let line = MiljoeDataClean {
            coordinates: [coord, coord],
            info: "Exact Match Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = correlation(points, lines);
        assert_eq!(results.len(), 1);
        assert!(results[0].relevant, "Exact match should be relevant");
    }

    // ============================================================================
    // TEST 6: Multiple Parking Zones - Closest one selected
    // ============================================================================
    #[test]
    fn test_multiple_zones_closest_selected() {
        let point = AdressClean {
            coordinates: [decimal("13.195000"), decimal("55.595000")],
            postnummer: "200 00".to_string(),
            adress: "Central Address".to_string(),
            gata: "Main Street".to_string(),
            gatunummer: "5".to_string(),
        };

        // Three parking zones at different distances
        let line1 = MiljoeDataClean {
            coordinates: [
                [decimal("13.190000"), decimal("55.590000")], // FAR (~700m)
                [decimal("13.190200"), decimal("55.590200")],
            ],
            info: "Far Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let line2 = MiljoeDataClean {
            coordinates: [
                [decimal("13.194800"), decimal("55.594800")], // CLOSEST (~20m)
                [decimal("13.195200"), decimal("55.595200")],
            ],
            info: "Closest Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let line3 = MiljoeDataClean {
            coordinates: [
                [decimal("13.200000"), decimal("55.600000")], // FAR (~700m)
                [decimal("13.200200"), decimal("55.600200")],
            ],
            info: "Other Far Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line1, line2, line3];

        let results = find_closest_lines(&points, &lines);
        let (closest_index, _) = results[0].unwrap();

        assert_eq!(
            closest_index, 1,
            "Should select the closest zone (index 1)"
        );
    }

    // ============================================================================
    // TEST 7: Correlation Output Structure Validation
    // ============================================================================
    #[test]
    fn test_correlation_output_structure() {
        let point = AdressClean {
            coordinates: [decimal("13.195000"), decimal("55.595000")],
            postnummer: "202 00".to_string(),
            adress: "Storgatan 15".to_string(),
            gata: "Storgatan".to_string(),
            gatunummer: "15".to_string(),
        };

        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.194800"), decimal("55.594800")],
                [decimal("13.195200"), decimal("55.595200")],
            ],
            info: "Parking Zone A".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        let results = correlation(points, lines);
        assert_eq!(results.len(), 1);

        let result = &results[0];
        assert!(result.relevant, "Should be marked relevant");
        assert_eq!(result.postnummer, "202 00");
        assert_eq!(result.adress, "Storgatan 15");
        assert_eq!(result.gata, "Storgatan");
        assert_eq!(result.gatunummer, "15");
        assert_eq!(result.info, "Parking Zone A");
        assert_eq!(result.tid, "08:00-18:00");
        assert_eq!(result.dag, 1);
    }

    // ============================================================================
    // TEST 8: Batch Processing - Multiple addresses and zones
    // ============================================================================
    #[test]
    fn test_multiple_addresses_batch_processing() {
        let points = vec![
            AdressClean {
                coordinates: [decimal("13.195000"), decimal("55.595000")],
                postnummer: "200 00".to_string(),
                adress: "Storgatan 1".to_string(),
                gata: "Storgatan".to_string(),
                gatunummer: "1".to_string(),
            },
            AdressClean {
                coordinates: [decimal("13.210000"), decimal("55.610000")],
                postnummer: "201 00".to_string(),
                adress: "Lilla Torg 5".to_string(),
                gata: "Lilla Torg".to_string(),
                gatunummer: "5".to_string(),
            },
            AdressClean {
                coordinates: [decimal("13.500000"), decimal("55.900000")], // VERY FAR
                postnummer: "202 00".to_string(),
                adress: "Västra Varvsgatan 10".to_string(),
                gata: "Västra Varvsgatan".to_string(),
                gatunummer: "10".to_string(),
            },
        ];

        let lines = vec![
            MiljoeDataClean {
                coordinates: [
                    [decimal("13.194800"), decimal("55.594800")],
                    [decimal("13.195200"), decimal("55.595200")],
                ],
                info: "Zone A".to_string(),
                tid: "08:00-18:00".to_string(),
                dag: 1,
            },
            MiljoeDataClean {
                coordinates: [
                    [decimal("13.209800"), decimal("55.609800")],
                    [decimal("13.210200"), decimal("55.610200")],
                ],
                info: "Zone B".to_string(),
                tid: "09:00-17:00".to_string(),
                dag: 2,
            },
        ];

        let results = correlation(points, lines);
        assert_eq!(results.len(), 3, "Should have 3 results");

        // First address should be relevant (close to Zone A)
        assert_eq!(results[0].adress, "Storgatan 1");
        assert!(results[0].relevant);
        assert_eq!(results[0].info, "Zone A");

        // Second address should be relevant (close to Zone B)
        assert_eq!(results[1].adress, "Lilla Torg 5");
        assert!(results[1].relevant);
        assert_eq!(results[1].info, "Zone B");

        // Third address is very far (not relevant)
        assert_eq!(results[2].adress, "Västra Varvsgatan 10");
        assert!(!results[2].relevant);
    }

    // ============================================================================
    // TEST 9: Real-world Malmö Coordinates
    // ============================================================================
    #[test]
    fn test_real_world_malmo_coordinates() {
        // Real Malmö addresses with actual coordinates
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

        let results = correlation(points, lines);
        assert_eq!(results.len(), 2);

        // Both should be relevant with real-world coordinates
        assert!(results[0].relevant, "Lilla Torg should be relevant");
        assert!(results[1].relevant, "Västra Varvsgatan should be relevant");

        // Verify correct zone associations
        assert_eq!(results[0].info, "Lilla Torg Miljözon");
        assert_eq!(results[1].info, "Västra Varvsgatan Miljözon");
    }

    // ============================================================================
    // TEST 10: Degenerate Line Segment (identical endpoints)
    // ============================================================================
    #[test]
    fn test_degenerate_line_segment_handling() {
        let point = AdressClean {
            coordinates: [decimal("13.195000"), decimal("55.595000")],
            postnummer: "200 00".to_string(),
            adress: "Test".to_string(),
            gata: "Test".to_string(),
            gatunummer: "1".to_string(),
        };

        // Degenerate segment (both endpoints identical)
        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.194800"), decimal("55.594800")],
                [decimal("13.194800"), decimal("55.594800")], // Same point
            ],
            info: "Point Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point];
        let lines = vec![line];

        // Should handle degenerate segment without panicking
        let results = find_closest_lines(&points, &lines);
        assert!(results[0].is_some(), "Should handle degenerate segment");
        assert!(results[0].unwrap().1 >= 0.0, "Distance should be non-negative");
    }

    // ============================================================================
    // TEST 11: Threshold Verification - 50 meter boundary
    // ============================================================================
    #[test]
    fn test_50m_threshold_boundary() {
        // Address in Malmö
        let point = AdressClean {
            coordinates: [decimal("13.195000"), decimal("55.595000")],
            postnummer: "200 00".to_string(),
            adress: "Test".to_string(),
            gata: "Test".to_string(),
            gatunummer: "1".to_string(),
        };

        // Parking zone ~25 meters away (within 50m)
        let line_near = MiljoeDataClean {
            coordinates: [
                [decimal("13.194800"), decimal("55.594800")], // ~25m away
                [decimal("13.195200"), decimal("55.595200")],
            ],
            info: "Near Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point.clone()];
        let lines = vec![line_near];

        let results = correlation(points.clone(), lines);
        assert!(
            results[0].relevant,
            "Zone within 50m should be relevant"
        );

        // Parking zone ~150 meters away (beyond 50m)
        let line_far = MiljoeDataClean {
            coordinates: [
                [decimal("13.192000"), decimal("55.592000")], // ~150m away
                [decimal("13.192200"), decimal("55.592200")],
            ],
            info: "Far Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let results = correlation(points, vec![line_far]);
        assert!(
            !results[0].relevant,
            "Zone beyond 50m should NOT be relevant"
        );
    }

    // ============================================================================
    // TEST 12: Performance - Batch processing with many records
    // ============================================================================
    #[test]
    fn test_batch_performance_many_records() {
        // Generate 100 test addresses scattered across Malmö
        let mut points = Vec::new();
        for i in 0..100 {
            let lat_offset = Decimal::from(i) * decimal("0.001");
            points.push(AdressClean {
                coordinates: [decimal("13.195000") + lat_offset, decimal("55.595000")],
                postnummer: "Malmö".to_string(),
                adress: format!("Address {}", i),
                gata: "Test Street".to_string(),
                gatunummer: format!("{}", i),
            });
        }

        // Generate 50 parking zones
        let mut lines = Vec::new();
        for i in 0..50 {
            let lat_offset = Decimal::from(i) * decimal("0.002");
            lines.push(MiljoeDataClean {
                coordinates: [
                    [decimal("13.195000") + lat_offset, decimal("55.595000")],
                    [decimal("13.195200") + lat_offset, decimal("55.595200")],
                ],
                info: format!("Zone {}", i),
                tid: "08:00-18:00".to_string(),
                dag: ((i % 7) as u8) + 1,
            });
        }

        let results = correlation(points, lines);
        assert_eq!(results.len(), 100, "Should return 100 results");

        // Should have at least some relevant addresses
        let relevant_count = results.iter().filter(|r| r.relevant).count();
        assert!(
            relevant_count > 0,
            "At least some addresses should be relevant"
        );
    }

    // ============================================================================
    // TEST 13: Distance Calculation Consistency
    // ============================================================================
    #[test]
    fn test_distance_calculation_consistency() {
        // Two identical queries should produce identical results
        let point = AdressClean {
            coordinates: [decimal("13.195000"), decimal("55.595000")],
            postnummer: "200 00".to_string(),
            adress: "Test Address".to_string(),
            gata: "Test Street".to_string(),
            gatunummer: "1".to_string(),
        };

        let line = MiljoeDataClean {
            coordinates: [
                [decimal("13.194000"), decimal("55.594000")],
                [decimal("13.196000"), decimal("55.596000")],
            ],
            info: "Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };

        let points = vec![point.clone()];
        let lines = vec![line.clone()];

        let results1 = find_closest_lines(&points, &lines);
        let results2 = find_closest_lines(&points, &lines);

        // Results should be identical
        assert_eq!(results1.len(), results2.len());
        if let (Some((idx1, dist1)), Some((idx2, dist2))) = (results1[0], results2[0]) {
            assert_eq!(idx1, idx2, "Same query should find same zone index");
            assert_eq!(dist1, dist2, "Distance should be identical");
        }
    }
}
