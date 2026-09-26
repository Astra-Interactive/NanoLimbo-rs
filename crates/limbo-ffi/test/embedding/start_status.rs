use crate::embedding::StartStatus;

#[test]
fn given_the_published_statuses_when_read_as_codes_then_they_keep_their_numbers() {
    assert_eq!(StartStatus::Ok.code(), 0);
    assert_eq!(StartStatus::NullToken.code(), 1);
    assert_eq!(StartStatus::InvalidConfigDirectory.code(), 2);
    assert_eq!(StartStatus::StartupFailed.code(), 3);
    assert_eq!(StartStatus::RuntimeUnavailable.code(), 4);
    assert_eq!(StartStatus::BindFailed.code(), 5);
}
