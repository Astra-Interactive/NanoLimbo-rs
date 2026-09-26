use crate::memory::MemorySource;
use crate::options::BenchTarget;

#[test]
fn given_a_target_without_a_pid_when_parsed_then_only_its_address_is_kept() {
    let target = BenchTarget::parse("rust=127.0.0.1:25565").expect("a valid target");

    assert_eq!(target.name, "rust");
    assert_eq!(target.address.port(), 25565);
    assert_eq!(target.memory, None);
}

#[test]
fn given_a_container_name_when_parsed_then_memory_comes_from_docker() {
    let target = BenchTarget::parse("rust=127.0.0.1:25565@nanolimbo-1").expect("valid");

    assert_eq!(
        target.memory,
        Some(MemorySource::Container {
            name: "nanolimbo-1".to_owned()
        })
    );
}

#[test]
fn given_a_target_with_a_pid_when_parsed_then_memory_can_be_reported_for_it() {
    let target = BenchTarget::parse("java=127.0.0.1:25577@4321").expect("a valid target");

    assert_eq!(target.memory, Some(MemorySource::Process { pid: 4321 }));
}

#[test]
fn given_an_ipv6_address_when_parsed_then_its_colons_are_not_mistaken_for_a_pid() {
    let target = BenchTarget::parse("rust=[::1]:25565@99").expect("a valid target");

    assert_eq!(target.address.port(), 25565);
    assert_eq!(target.memory, Some(MemorySource::Process { pid: 99 }));
}

#[test]
fn given_something_that_is_not_a_target_when_parsed_then_it_is_rejected() {
    assert!(BenchTarget::parse("no-equals-sign").is_err());
    assert!(BenchTarget::parse("rust=not-an-address").is_err());
    assert!(BenchTarget::parse("rust=127.0.0.1:1@").is_err());
}
