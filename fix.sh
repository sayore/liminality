sed -i '/fn test_warp_connects_different_spaces/,$d' crates/liminality-model/src/lib.rs
sed -i '/^}$/d' crates/liminality-model/src/lib.rs
cat << 'INNER_EOF' >> crates/liminality-model/src/lib.rs

    #[test]
    fn test_warp_connects_different_spaces() {
        use crate::node::{NodeKind, WarpNode};

        let space1 = SpaceId::from("dimension_alpha");
        let space2 = SpaceId::from("dimension_beta");

        let target_pos = SpacePos::new(space2.clone(), 0, 0, 0);

        let warp = WarpNode {
            target: target_pos.clone(),
            latency: None,
            throughput: None,
            filter: ResourceFilter::Any,
        };

        let node_kind = NodeKind::Warp(warp);

        if let NodeKind::Warp(w) = node_kind {
            assert_ne!(space1, w.target.w);
            assert_eq!(space2, w.target.w);
            assert_eq!(target_pos, w.target);
        } else {
            panic!("Expected WarpNode");
        }
    }
}
INNER_EOF
