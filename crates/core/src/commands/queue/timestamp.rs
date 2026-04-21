use crate::prelude::*;
#[cfg(test)]
use chrono::ParseError;
use chrono::{DateTime, SecondsFormat, Utc};

/// UTC timestamp for queue item status tracking.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TimeStamp {
    datetime: DateTime<Utc>,
}

impl TimeStamp {
    /// Create a [`TimeStamp`] for the current UTC time.
    pub(crate) fn now() -> Self {
        TimeStamp {
            datetime: Utc::now(),
        }
    }

    /// Create a [`TimeStamp`] from an RFC 3339 date string.
    #[cfg(test)]
    pub(crate) fn from_rfc3339(s: &str) -> Result<Self, ParseError> {
        let datetime = DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc);
        Ok(TimeStamp { datetime })
    }
}

impl Serialize for TimeStamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.datetime.to_rfc3339_opts(SecondsFormat::Secs, true);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for TimeStamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimeStampVisitor;

        impl Visitor<'_> for TimeStampVisitor {
            type Value = TimeStamp;
            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("a valid ISO 8601 date string")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                let datetime = DateTime::from_str(value).map_err(SerdeError::custom)?;
                Ok(TimeStamp { datetime })
            }
        }
        deserializer.deserialize_str(TimeStampVisitor)
    }
}
