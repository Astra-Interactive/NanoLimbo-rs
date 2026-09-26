use crate::settings::network::TransportType;

#[test]
fn given_every_spelling_in_the_shipped_comment_when_parsed_then_all_are_recognised() {
    for name in ["EPOLL", "IO_URING", "KQUEUE", "NIO"] {
        assert_eq!(
            name.parse::<TransportType>()
                .map(TransportType::config_name)
                .ok(),
            Some(name)
        );
    }
}

#[test]
fn given_a_transport_that_is_not_offered_when_parsed_then_it_is_rejected() {
    assert!("SPINNING_RUST".parse::<TransportType>().is_err());
}

#[test]
fn given_io_uring_when_asked_whether_the_runtime_offers_it_then_it_does_not() {
    assert!(!TransportType::IoUring.is_available());
    assert!(TransportType::Epoll.is_available());
    assert!(TransportType::KQueue.is_available());
    assert!(TransportType::Nio.is_available());
}
