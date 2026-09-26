use crate::memory::MemorySource;

#[test]
fn given_digits_when_read_then_they_name_a_process() {
    assert_eq!(
        MemorySource::parse("4321"),
        MemorySource::Process { pid: 4321 }
    );
}

#[test]
fn given_a_name_when_read_then_it_names_a_container() {
    assert_eq!(
        MemorySource::parse("nanolimbo-1"),
        MemorySource::Container {
            name: "nanolimbo-1".to_owned()
        }
    );
}
