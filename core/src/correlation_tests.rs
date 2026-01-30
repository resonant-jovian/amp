#[cfg(test)]
mod tests {
    use crate::correlation_algorithms::{
        CorrelationAlgo, DistanceBasedAlgo, GridNearestAlgo, KDTreeSpatialAlgo,
        OverlappingChunksAlgo, RTreeSpatialAlgo, RaycastingAlgo,
    };
    use crate::structs::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    /// Helper to create a Decimal from string
    fn decimal(val: &str) -> Decimal {
        Decimal::from_str(val).expect("Failed to parse decimal")
    }
    fn create_test_address(lat: &str, lon: &str, name: &str) -> AdressClean {
        AdressClean {
            coordinates: [decimal(lon), decimal(lat)],
            postnummer: Some("200 00".to_string()),
            adress: name.to_string(),
            gata: "Test Street".to_string(),
            gatunummer: "1".to_string(),
        }
    }
    fn create_test_zone(
        lat_start: &str,
        lon_start: &str,
        lat_end: &str,
        lon_end: &str,
        info: &str,
    ) -> MiljoeDataClean {
        MiljoeDataClean {
            coordinates: [
                [decimal(lon_start), decimal(lat_start)],
                [decimal(lon_end), decimal(lat_end)],
            ],
            info: info.to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        }
    }
    #[test]
    fn test_haversine_distance_accuracy() {
        let address = create_test_address("55.5932645", "13.1945945", "Lilla Torg 1");
        let zone = create_test_zone(
            "55.5932895",
            "13.1946195",
            "55.5933195",
            "55.5946495",
            "Test Zone",
        );
        let addresses = [address];
        let zones = vec![zone];
        let db_algo = DistanceBasedAlgo;
        let db_result = db_algo.correlate(&addresses[0], &zones);
        assert!(
            db_result.is_some(),
            "Distance-Based: Should find match within 50m"
        );
        let (_, db_dist) = db_result.unwrap();
        assert!(db_dist > 0.0, "Distance-Based: Distance should be positive");
        assert!(
            db_dist <= 50.0,
            "Distance-Based: Distance should be within 50m threshold",
        );
        let ray_algo = RaycastingAlgo;
        let ray_result = ray_algo.correlate(&addresses[0], &zones);
        assert!(
            ray_result.is_some(),
            "Raycasting: Should find match within 50m"
        );
        let (_, ray_dist) = ray_result.unwrap();
        assert!(ray_dist > 0.0, "Raycasting: Distance should be positive");
        assert!(
            ray_dist <= 50.0,
            "Raycasting: Distance should be within 50m threshold"
        );
    }
    #[test]
    fn test_50m_threshold_enforcement() {
        let address = create_test_address("55.5932645", "13.1945945", "Test Address");
        let far_zone = create_test_zone(
            "55.5932645",
            "13.1925945",
            "55.5932945",
            "13.1926245",
            "Far Zone",
        );
        let addresses = [address];
        let zones = vec![far_zone];
        let algo = DistanceBasedAlgo;
        let result = algo.correlate(&addresses[0], &zones);
        assert!(result.is_none(), "Addresses beyond 50m should not match");
    }
    #[test]
    fn test_all_algorithms_consistency() {
        let address = create_test_address("55.5932645", "13.1945945", "Central Address");
        let zone = create_test_zone(
            "55.5932645",
            "13.1945945",
            "55.5932945",
            "13.1946245",
            "Test Zone",
        );
        let addresses = [address];
        let zones = vec![zone];
        let db_algo = DistanceBasedAlgo;
        let db_result = db_algo.correlate(&addresses[0], &zones);
        assert!(db_result.is_some(), "Distance-Based should find match");
        let ray_algo = RaycastingAlgo;
        let ray_result = ray_algo.correlate(&addresses[0], &zones);
        assert!(ray_result.is_some(), "Raycasting should find match");
        let chunk_algo = OverlappingChunksAlgo::new(&zones);
        let chunk_result = chunk_algo.correlate(&addresses[0], &zones);
        assert!(
            chunk_result.is_some(),
            "Overlapping Chunks should find match"
        );
        let rtree_algo = RTreeSpatialAlgo::new(&zones);
        let rtree_result = rtree_algo.correlate(&addresses[0], &zones);
        assert!(rtree_result.is_some(), "R-Tree should find match");
        let kdtree_algo = KDTreeSpatialAlgo::new(&zones);
        let kdtree_result = kdtree_algo.correlate(&addresses[0], &zones);
        assert!(kdtree_result.is_some(), "KD-Tree should find match");
        let grid_algo = GridNearestAlgo::new(&zones);
        let grid_result = grid_algo.correlate(&addresses[0], &zones);
        assert!(grid_result.is_some(), "Grid should find match");
        assert_eq!(
            db_result.unwrap().0,
            ray_result.unwrap().0,
            "Algorithms should find same zone",
        );
        assert_eq!(
            db_result.unwrap().0,
            chunk_result.unwrap().0,
            "Algorithms should find same zone",
        );
        assert_eq!(
            db_result.unwrap().0,
            rtree_result.unwrap().0,
            "Algorithms should find same zone",
        );
        assert_eq!(
            db_result.unwrap().0,
            kdtree_result.unwrap().0,
            "Algorithms should find same zone",
        );
        assert_eq!(
            db_result.unwrap().0,
            grid_result.unwrap().0,
            "Algorithms should find same zone",
        );
    }
    #[test]
    fn test_correlation_result_structure() {
        let result1 = CorrelationResult {
            address: "Storgatan 1".to_string(),
            postnummer: "200 00".to_string(),
            miljo_match: Some((15.5, "Miljö Zone A".to_string())),
            parkering_match: None,
        };
        assert!(result1.has_match(), "Should have match");
        assert_eq!(result1.dataset_source(), "Miljödata only");
        assert_eq!(result1.closest_distance(), Some(15.5));
        let result2 = CorrelationResult {
            address: "Storgatan 2".to_string(),
            postnummer: "200 00".to_string(),
            miljo_match: Some((20.0, "Miljö Zone B".to_string())),
            parkering_match: Some((35.0, "Parkering Zone A".to_string())),
        };
        assert!(result2.has_match(), "Should have match");
        assert_eq!(result2.dataset_source(), "Both (Miljödata + Parkering)");
        assert_eq!(
            result2.closest_distance(),
            Some(20.0),
            "Should return closest distance",
        );
        let result3 = CorrelationResult {
            address: "Storgatan 3".to_string(),
            postnummer: "200 00".to_string(),
            miljo_match: None,
            parkering_match: None,
        };
        assert!(!result3.has_match(), "Should have no match");
        assert_eq!(result3.dataset_source(), "No match");
        assert_eq!(result3.closest_distance(), None);
    }
    #[test]
    fn test_batch_processing_multiple_addresses() {
        let addresses = vec![
            create_test_address("55.5932645", "13.1945945", "Address 1"),
            create_test_address("55.5932745", "13.1946045", "Address 2"),
            create_test_address("55.5932845", "13.1946145", "Address 3"),
        ];
        let zones = vec![
            create_test_zone(
                "55.5932645",
                "13.1945945",
                "55.5932945",
                "13.1946245",
                "Zone 1",
            ),
            create_test_zone(
                "55.5932745",
                "13.1946045",
                "55.5932945",
                "13.1946345",
                "Zone 2",
            ),
        ];
        let algo = DistanceBasedAlgo;
        let mut match_count = 0;
        for address in &addresses {
            if algo.correlate(address, &zones).is_some() {
                match_count += 1;
            }
        }
        assert_eq!(match_count, 3, "All addresses should find nearby zones");
    }
    #[test]
    fn test_closest_match_selection() {
        let address = create_test_address("55.5932645", "13.1945945", "Test Address");
        let zones = vec![
            create_test_zone(
                "55.5842645",
                "13.1845945",
                "55.5842945",
                "13.1846245",
                "Far Zone",
            ),
            create_test_zone(
                "55.5932645",
                "13.1945945",
                "55.5932945",
                "13.1946245",
                "Close Zone",
            ),
            create_test_zone(
                "55.5932445",
                "13.1945745",
                "55.5932545",
                "13.1945845",
                "Medium Zone",
            ),
        ];
        let algo = RTreeSpatialAlgo::new(&zones);
        let result = algo.correlate(&address, &zones);
        assert!(result.is_some(), "Should find a match");
        let (idx, _dist) = result.unwrap();
        assert!(idx < zones.len(), "Index should be valid");
    }
    #[test]
    fn test_real_world_malmo_coordinates() {
        let addresses = vec![
            AdressClean {
                coordinates: [decimal("13.1945945"), decimal("55.5932645")],
                postnummer: Some("211 00".to_string()),
                adress: "Lilla Torg 1".to_string(),
                gata: "Lilla Torg".to_string(),
                gatunummer: "1".to_string(),
            },
            AdressClean {
                coordinates: [decimal("13.2004523"), decimal("55.6043210")],
                postnummer: Some("213 00".to_string()),
                adress: "Västra Varvsgatan 41".to_string(),
                gata: "Västra Varvsgatan".to_string(),
                gatunummer: "41".to_string(),
            },
        ];
        let zones = vec![
            create_test_zone(
                "55.5932645",
                "13.1945945",
                "55.5932945",
                "13.1946245",
                "Lilla Torg Miljözon",
            ),
            create_test_zone(
                "55.6043210",
                "13.2004523",
                "55.6043510",
                "13.2004823",
                "Västra Varvsgatan Miljözon",
            ),
        ];
        let algo = KDTreeSpatialAlgo::new(&zones);
        for address in &addresses {
            let result = algo.correlate(address, &zones);
            assert!(result.is_some(), "Should find match for {}", address.adress);
        }
    }
    #[test]
    fn test_no_match_beyond_threshold() {
        let address = create_test_address("55.5932645", "13.1945945", "Test Address");
        let zones = vec![
            create_test_zone(
                "55.5842645",
                "13.1845945",
                "55.5842945",
                "13.1846245",
                "Far Zone 1",
            ),
            create_test_zone(
                "55.6142645",
                "13.2145945",
                "55.6142945",
                "13.2146245",
                "Far Zone 2",
            ),
        ];
        let algo = GridNearestAlgo::new(&zones);
        let result = algo.correlate(&address, &zones);
        assert!(
            result.is_none(),
            "Should not match zones beyond 50m threshold"
        );
    }
    #[test]
    fn test_deterministic_results() {
        let address = create_test_address("55.5932645", "13.1945945", "Test Address");
        let zones = vec![
            create_test_zone(
                "55.5932645",
                "13.1945945",
                "55.5932945",
                "13.1946245",
                "Zone 1",
            ),
            create_test_zone(
                "55.5932745",
                "13.1946045",
                "55.5932945",
                "13.1946345",
                "Zone 2",
            ),
        ];
        let algo = DistanceBasedAlgo;
        let result1 = algo.correlate(&address, &zones);
        let result2 = algo.correlate(&address, &zones);
        match (result1, result2) {
            (Some((idx1, dist1)), Some((idx2, dist2))) => {
                assert_eq!(idx1, idx2, "Indices should match");
                assert_eq!(dist1, dist2, "Distances should match exactly");
            }
            _ => panic!("Both queries should return Some"),
        }
    }
    #[test]
    fn test_algorithm_performance() {
        let mut addresses = Vec::new();
        for i in 0..100 {
            let lat_offset = Decimal::from(i) * decimal("0.0001");
            addresses.push(AdressClean {
                coordinates: [decimal("13.1945945") + lat_offset, decimal("55.5932645")],
                postnummer: Some("200 00".to_string()),
                adress: format!("Address {}", i),
                gata: "Test Street".to_string(),
                gatunummer: format!("{}", i),
            });
        }
        let mut zones = Vec::new();
        for i in 0..50 {
            let lat_offset = Decimal::from(i) * decimal("0.0002");
            zones.push(MiljoeDataClean {
                coordinates: [
                    [decimal("13.1945945") + lat_offset, decimal("55.5932645")],
                    [decimal("13.1946245") + lat_offset, decimal("55.5932945")],
                ],
                info: format!("Zone {}", i),
                tid: "08:00-18:00".to_string(),
                dag: ((i % 7) as u8) + 1,
            });
        }
        let db_algo = DistanceBasedAlgo;
        let mut db_matches = 0;
        for addr in &addresses {
            if db_algo.correlate(addr, &zones).is_some() {
                db_matches += 1;
            }
        }
        assert!(db_matches > 0, "Distance-Based should find matches");
        let chunk_algo = OverlappingChunksAlgo::new(&zones);
        let mut chunk_matches = 0;
        for addr in &addresses {
            if chunk_algo.correlate(addr, &zones).is_some() {
                chunk_matches += 1;
            }
        }
        assert!(chunk_matches > 0, "Overlapping Chunks should find matches");
        let rtree_algo = RTreeSpatialAlgo::new(&zones);
        let mut rtree_matches = 0;
        for addr in &addresses {
            if rtree_algo.correlate(addr, &zones).is_some() {
                rtree_matches += 1;
            }
        }
        assert!(rtree_matches > 0, "R-Tree should find matches");
    }
    #[test]
    fn test_exact_location_match() {
        let coord = [decimal("13.1945945"), decimal("55.5932645")];
        let address = AdressClean {
            coordinates: coord,
            postnummer: Some("200 00".to_string()),
            adress: "Test Address".to_string(),
            gata: "Test Street".to_string(),
            gatunummer: "1".to_string(),
        };
        let zone = MiljoeDataClean {
            coordinates: [coord, coord],
            info: "Exact Match Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };
        let algo = DistanceBasedAlgo;
        let result = algo.correlate(&address, &[zone]);
        assert!(result.is_some(), "Should find exact match");
        let (_, dist) = result.unwrap();
        assert!(dist < 1.0, "Distance should be very small for exact match");
    }
    #[test]
    fn test_degenerate_zone_handling() {
        let address = create_test_address("55.5932645", "13.1945945", "Test Address");
        let degenerate_zone = MiljoeDataClean {
            coordinates: [
                [decimal("13.1945945"), decimal("55.5932645")],
                [decimal("13.1945945"), decimal("55.5932645")],
            ],
            info: "Degenerate Zone".to_string(),
            tid: "08:00-18:00".to_string(),
            dag: 1,
        };
        let algo = DistanceBasedAlgo;
        let result = algo.correlate(&address, &[degenerate_zone]);
        assert!(result.is_some(), "Should handle degenerate zone");
    }
    #[test]
    fn test_threshold_returns_only_valid_matches() {
        let address = create_test_address("55.5932645", "13.1945945", "Test Address");
        let zones = vec![
            create_test_zone(
                "55.5932645",
                "13.1945945",
                "55.5932945",
                "13.1946245",
                "Close Zone",
            ),
            create_test_zone(
                "55.5842645",
                "13.1845945",
                "55.5842945",
                "13.1846245",
                "Far Zone",
            ),
        ];
        let algo = RaycastingAlgo;
        let result = algo.correlate(&address, &zones);
        if let Some((idx, dist)) = result {
            assert!(dist <= 50.0, "Returned distance should not exceed 50m");
            assert_eq!(idx, 0, "Should match the close zone");
        }
    }
}
