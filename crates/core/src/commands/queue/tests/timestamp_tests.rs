use crate::testing_prelude::*;

#[test]
fn test_serialize_timestamp() {
    let timestamp = TimeStamp::from_rfc3339("2024-10-18T12:34:56+01:30").unwrap();
    let serialized = serde_json::to_string(&timestamp).unwrap();
    assert_eq!(serialized, "\"2024-10-18T11:04:56Z\"");
}

#[test]
fn test_serialize_timestamp_zulu() {
    let timestamp = TimeStamp::from_rfc3339("2024-10-18T12:34:56Z").unwrap();
    let serialized = serde_json::to_string(&timestamp).unwrap();
    assert_eq!(serialized, "\"2024-10-18T12:34:56Z\"");
}

#[test]
fn deserialize_timestamp() {
    // Arrange
    let json = "\"2024-10-18T12:34:56-02:30\"";
    let deserialized: TimeStamp = serde_json::from_str(json).unwrap();

    // Act
    let expected = TimeStamp::from_rfc3339("2024-10-18T12:34:56-02:30").unwrap();

    // Assert
    assert_eq!(deserialized, expected);
}

#[test]
fn round_trip_serialization() {
    // Arrange
    let timestamp = TimeStamp::from_rfc3339("2024-10-18T12:34:56-02:30").unwrap();

    // Act
    let serialized = serde_json::to_string(&timestamp).unwrap();
    let deserialized: TimeStamp = serde_json::from_str(&serialized).unwrap();

    // Assert
    assert_eq!(timestamp, deserialized);
}

#[test]
fn invalid_timestamp_format() {
    // Arrange
    let invalid_json = "\"invalid-date-format\"";

    // Act
    let result: Result<TimeStamp, _> = serde_json::from_str(invalid_json);

    // Assert
    assert!(
        result.is_err(),
        "Deserialization should fail with invalid date format"
    );
}
