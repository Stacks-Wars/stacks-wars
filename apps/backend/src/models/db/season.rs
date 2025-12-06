use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents gameplay seasons for leaderboard resets and rewards
/// Maps to `seasons` table in PostgreSQL
///
/// Seasons define time-bound competitive periods where players compete
/// for rankings and rewards. Each season has its own leaderboard tracked
/// via `user_wars_points`.
///
/// # Database Schema
/// - Primary key: `id` (auto-increment)
/// - Constraints: end_date must be after start_date
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl Season {
    /// Validate that end_date is after start_date.
    pub fn validate_date_range(
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<(NaiveDateTime, NaiveDateTime), DateRangeError> {
        if end_date <= start_date {
            return Err(DateRangeError::EndBeforeStart {
                start: start_date,
                end: end_date,
            });
        }
        Ok((start_date, end_date))
    }

    /// Parse and validate date strings in format "YYYY-MM-DD HH:MM:SS".
    pub fn parse_date_range(
        start: &str,
        end: &str,
    ) -> Result<(NaiveDateTime, NaiveDateTime), DateRangeError> {
        let start_date =
            NaiveDateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S").map_err(|e| {
                DateRangeError::InvalidFormat {
                    field: "start_date".to_string(),
                    value: start.to_string(),
                    error: e.to_string(),
                }
            })?;

        let end_date = NaiveDateTime::parse_from_str(end, "%Y-%m-%d %H:%M:%S").map_err(|e| {
            DateRangeError::InvalidFormat {
                field: "end_date".to_string(),
                value: end.to_string(),
                error: e.to_string(),
            }
        })?;

        Self::validate_date_range(start_date, end_date)
    }
}

/// Date range validation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DateRangeError {
    #[error("End date ({end}) must be after start date ({start})")]
    EndBeforeStart {
        start: NaiveDateTime,
        end: NaiveDateTime,
    },

    #[error("Invalid {field} format: '{value}'. Expected YYYY-MM-DD HH:MM:SS. Error: {error}")]
    InvalidFormat {
        field: String,
        value: String,
        error: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(year: i32, month: u32, day: u32, hour: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, min, 0)
            .unwrap()
    }

    #[test]
    fn test_validate_date_range_valid() {
        let start = date(2024, 1, 1, 0, 0);
        let end = date(2024, 12, 31, 23, 59);

        let result = Season::validate_date_range(start, end);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (start, end));
    }

    #[test]
    fn test_validate_date_range_end_before_start() {
        let start = date(2024, 12, 31, 23, 59);
        let end = date(2024, 1, 1, 0, 0);

        let result = Season::validate_date_range(start, end);
        assert!(matches!(result, Err(DateRangeError::EndBeforeStart { .. })));
    }

    #[test]
    fn test_validate_date_range_equal() {
        let date = date(2024, 6, 15, 12, 0);
        let result = Season::validate_date_range(date, date);
        assert!(matches!(result, Err(DateRangeError::EndBeforeStart { .. })));
    }

    #[test]
    fn test_parse_date_range_valid() {
        let result = Season::parse_date_range("2024-01-01 00:00:00", "2024-12-31 23:59:00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_date_range_invalid_format() {
        let result = Season::parse_date_range("2024/01/01 00:00:00", "2024-12-31 23:59:00");
        assert!(matches!(result, Err(DateRangeError::InvalidFormat { .. })));
    }

    #[test]
    fn test_parse_date_range_end_before_start() {
        let result = Season::parse_date_range("2024-12-31 23:59:00", "2024-01-01 00:00:00");
        assert!(matches!(result, Err(DateRangeError::EndBeforeStart { .. })));
    }
}
