#[cfg(test)]
mod ops_board_tests {
    use super::*;

    /// Each lane offers only the moves that fit the parcels in it: the
    /// office bags or sends out only what it still holds; delivered /
    /// failed only for what is out; failed parcels go out again or back.
    #[test]
    fn each_lane_offers_only_its_moves() {
        use manifest::OpsLane::*;
        let d = ops_lane_acts(ToDispatch);
        assert!(d.dispatch && !d.deliver && !d.fail && !d.failed);
        let c = ops_lane_acts(WithCarrier);
        assert!(!c.dispatch && c.carrier_update && c.deliver && c.fail && !c.failed);
        let o = ops_lane_acts(OutForDelivery);
        assert!(!o.dispatch && !o.carrier_update && o.deliver && o.fail && !o.failed);
        let f = ops_lane_acts(Failed);
        assert!(!f.dispatch && !f.deliver && !f.fail && f.failed && f.reasons);
        assert_eq!(ops_lane_acts(Done), OpsActs::default());
    }

    /// The pill a card wears follows its lane's colour.
    #[test]
    fn a_card_pill_matches_its_lane() {
        use manifest::TrackingStatus as T;
        for st in T::all() {
            let slot = ops_pill_slot(*st);
            let same_lane = T::all()
                .iter()
                .filter(|o| o.lane() == st.lane() && o.lane() != manifest::OpsLane::Done)
                .all(|o| ops_pill_slot(*o) == slot);
            assert!(same_lane, "{st:?}");
        }
        assert_ne!(ops_pill_slot(T::Delivered), ops_pill_slot(T::Rto));
    }
}
