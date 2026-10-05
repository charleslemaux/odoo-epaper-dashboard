use dashboard_core::gfx::{Frame, BUFFER_SIZE};

#[test]
fn frame_matches_c_renderer_byte_for_byte() {
    let fixture: &[u8] = include_bytes!("fixtures/preview_c.raw");
    assert_eq!(fixture.len(), BUFFER_SIZE);
    let mut fb: Box<Frame> = Box::new([0u8; BUFFER_SIZE]);
    host_tools::render_sample(&mut fb);
    if fb[..] != *fixture {
        let first = fb.iter().zip(fixture).position(|(a, b)| a != b).unwrap();
        let (x, y) = ((first % 400) * 2, first / 400);
        panic!(
            "frame differs from C render at byte {first} (approx pixel x={x}, y={y}): rust=0x{:02x} c=0x{:02x}",
            fb[first], fixture[first]
        );
    }
}
