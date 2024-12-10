use chrono::Utc;

pub fn now_utc_millis() -> i64 {
    Utc::now().timestamp_millis()
}
