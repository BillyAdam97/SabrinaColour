use super::{Bus, IE, IF};

#[test]
fn pending_interrupts_requires_matching_enable_and_request_bits() {
    let mut bus = Bus::new();

    for (enabled, requested, pending) in [
        (0x00, 0x1F, 0x00),
        (0x1F, 0x00, 0x00),
        (0x05, 0x0A, 0x00),
        (0x15, 0x1C, 0x14),
        (0x1F, 0x1F, 0x1F),
        (0xE0, 0xE0, 0x00),
        (0xFF, 0xFF, 0x1F),
    ] {
        bus.write(IE, enabled);
        bus.write(IF, requested);

        assert_eq!(
            bus.pending_interrupts(),
            pending,
            "IE={enabled:02X}, IF={requested:02X}"
        );
        assert_eq!(bus.read(IE), enabled);
        assert_eq!(bus.read(IF), requested);
    }
}
