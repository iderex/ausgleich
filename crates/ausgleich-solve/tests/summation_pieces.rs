//! Covers the deliberate violation this branch carries, so the branch reds the
//! invariants check alone and leaves the coverage floor where it is. A fixture
//! that reds two checks does not say which of them it is about.

#[test]
fn the_split_reports_at_least_one_piece() {
    assert!(ausgleich_solve::summation_pieces() >= 1);
}
