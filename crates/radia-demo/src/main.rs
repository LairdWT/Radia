fn main() {
    assert!(
        radia_render::crate_ready(),
        "Radia workspace failed to link"
    );
    println!("Radia bootstrap ready");
}
