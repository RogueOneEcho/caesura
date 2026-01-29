use chrono::{DateTime, Utc};
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// UTC timestamp for queue item status tracking.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TimeStamp {
    datetime: DateTime<Utc>,
}

impl TimeStamp {
    pub(crate) fn now() -> Self {
        TimeStamp {
            datetime: Utc::now(),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_rfc3339(s: &str) -> Result<Self, chrono::ParseError> {
        let datetime = DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc);
        Ok(TimeStamp { datetime })
    }
}

impl Serialize for TimeStamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self
            .datetime
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for TimeStamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TimeStampVisitor;

        impl Visitor<'_> for TimeStampVisitor {
            type Value = TimeStamp;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid ISO 8601 date string")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let datetime = DateTime::from_str(value).map_err(de::Error::custom)?;
                Ok(TimeStamp { datetime })
            }
        }
        deserializer.deserialize_str(TimeStampVisitor)
    }
}
