pub mod cli;
pub mod rules;

#[cfg(test)]
mod external_test;

#[cfg(test)]
mod production_test;

use once_cell::sync::Lazy;
use rules::{LogcheckDatabase, RuleCategory};
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::os::raw::c_char;

/// Log entry structure matching the Python LogEntry model
#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    #[serde(rename = "@timestamp")]
    timestamp: String,
    #[serde(rename = "MESSAGE")]
    message: String,
    #[serde(rename = "PRIORITY")]
    priority: Option<String>,
    #[serde(rename = "SYSTEMD_UNIT")]
    systemd_unit: Option<String>,
    #[serde(rename = "PID")]
    pid: Option<String>,
    #[serde(rename = "HOSTNAME")]
    hostname: Option<String>,
    #[serde(rename = "SYSLOG_IDENTIFIER")]
    syslog_identifier: Option<String>,
}

/// Initialize logcheck database with embedded rules
/// In production, this would load from /etc/logcheck/ or similar
static LOGCHECK_DB: Lazy<LogcheckDatabase> = Lazy::new(|| {
    let mut db = LogcheckDatabase::new();

    // Add some basic embedded rules for demonstration
    // In production, you'd call db.load_from_directory("/etc/logcheck")

    // Security violation rules
    let _ = db
        .violations_rules
        .add_pattern("^.*authentication failure.*$".to_string());
    let _ = db
        .violations_rules
        .add_pattern("^.*[Ff]ailed password.*$".to_string());
    let _ = db
        .violations_rules
        .add_pattern("^.*sudo.*authentication failure.*$".to_string());
    let _ = db
        .violations_rules
        .add_pattern("^.*segfault|segmentation fault.*$".to_string());
    let _ = db
        .violations_rules
        .add_pattern("^.*Out of memory|OOM.*$".to_string());

    // System events to ignore (normal operations)
    let _ = db
        .server
        .add_pattern("^.*Started Session \\d+ of user.*$".to_string());
    let _ = db
        .server
        .add_pattern("^.*Stopped Session \\d+ of user.*$".to_string());
    let _ = db
        .server
        .add_pattern("^.*systemd-logind.*: New session.*of user.*$".to_string());
    let _ = db
        .server
        .add_pattern("^.*systemd-logind.*: Session.*logged out.*$".to_string());
    let _ = db
        .server
        .add_pattern("^.*sshd.*: Accepted (password|publickey) for.*$".to_string());
    let _ = db
        .server
        .add_pattern("^.*sshd.*: Connection closed by.*$".to_string());

    // Compile all rule sets
    let _ = db.compile_all();

    db
});

/// Apply logcheck rules to a log entry using the modern rule database
fn apply_logcheck_rules(log_entry: &LogEntry) -> Option<RuleCategory> {
    LOGCHECK_DB.match_message(&log_entry.message)
}

/// Create a filtered log entry with logcheck metadata
fn create_filtered_entry(log_entry: &LogEntry, matched_category: Option<RuleCategory>) -> String {
    let mut filtered_entry = serde_json::to_value(log_entry).unwrap();

    if let Some(category) = matched_category {
        let (rule_type, description) = match category {
            RuleCategory::Cracking => ("cracking", "Active intrusion attempt detected"),
            RuleCategory::CrackingIgnore => (
                "cracking_ignore",
                "Known false positive for cracking detection",
            ),
            RuleCategory::Violations => {
                ("violations", "Security violation or critical system event")
            }
            RuleCategory::ViolationsIgnore => (
                "violations_ignore",
                "Known false positive for security violation",
            ),
            RuleCategory::SystemEvents => ("ignore", "Normal system event"),
            RuleCategory::Workstation => ("ignore", "Normal workstation event"),
            RuleCategory::Server => ("ignore", "Normal server event"),
            RuleCategory::Local => ("ignore", "Local custom rule match"),
        };

        filtered_entry["logcheck_rule_type"] = serde_json::Value::String(rule_type.to_string());
        filtered_entry["logcheck_description"] = serde_json::Value::String(description.to_string());
        filtered_entry["logcheck_matched"] = serde_json::Value::Bool(true);

        // Add category for more detailed classification
        filtered_entry["logcheck_category"] = serde_json::Value::String(format!("{:?}", category));
    } else {
        filtered_entry["logcheck_matched"] = serde_json::Value::Bool(false);
        filtered_entry["logcheck_rule_type"] =
            serde_json::Value::String("unclassified".to_string());
        filtered_entry["logcheck_description"] =
            serde_json::Value::String("No matching logcheck rule found".to_string());
    }

    serde_json::to_string(&filtered_entry).unwrap_or_else(|_| "{}".to_string())
}

/// Parse input record as JSON and extract log entry
fn parse_log_entry(record: *const c_char, record_len: usize) -> Option<LogEntry> {
    if record.is_null() || record_len == 0 {
        return None;
    }

    let record_slice = unsafe { std::slice::from_raw_parts(record as *const u8, record_len) };
    let record_str = std::str::from_utf8(record_slice).ok()?;

    serde_json::from_str::<LogEntry>(record_str).ok()
}

/// Convert Rust string to leaked C string for Fluent-Bit
fn to_c_string(s: String) -> *const c_char {
    let boxed = CString::new(s)
        .unwrap_or_else(|_| CString::new("{}").unwrap())
        .into_boxed_c_str();
    Box::leak(boxed).as_ptr()
}

/// Main logcheck filter function for JSON format
/// Returns: Modified JSON record with logcheck metadata, or NULL to drop the record
#[unsafe(no_mangle)]
pub extern "C" fn logcheck_filter_json(
    _tag: *const c_char,
    _tag_len: usize,
    _time_sec: u32,
    _time_nsec: u32,
    record: *const c_char,
    record_len: usize,
) -> *const c_char {
    // Parse the incoming log entry
    let log_entry = match parse_log_entry(record, record_len) {
        Some(entry) => entry,
        None => {
            // Return original record if parsing fails
            let record_slice =
                unsafe { std::slice::from_raw_parts(record as *const u8, record_len) };
            let record_str = std::str::from_utf8(record_slice).unwrap_or("{}");
            return to_c_string(record_str.to_string());
        }
    };

    // Apply logcheck rules
    let matched_category = apply_logcheck_rules(&log_entry);

    // For now, let all messages through but add metadata
    // In a production setup, you might want to drop "ignore" messages entirely
    let filtered_json = create_filtered_entry(&log_entry, matched_category);

    to_c_string(filtered_json)
}

/// Test/demo function that adds simple metadata
#[unsafe(no_mangle)]
pub extern "C" fn logcheck_demo_json(
    _tag: *const c_char,
    _tag_len: usize,
    time_sec: u32,
    _time_nsec: u32,
    _record: *const c_char,
    _record_len: usize,
) -> *const c_char {
    let demo_msg = format!(
        r#"{{"logcheck_demo":"Hello from Rust WASM logcheck filter!","timestamp":{},"processed":true}}"#,
        time_sec
    );

    to_c_string(demo_msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_violation_matching() {
        let log_entry = LogEntry {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            message: "sshd[1234]: Failed password for root from 192.168.1.1 port 22".to_string(),
            priority: Some("4".to_string()),
            systemd_unit: Some("ssh.service".to_string()),
            pid: Some("1234".to_string()),
            hostname: Some("testhost".to_string()),
            syslog_identifier: Some("sshd".to_string()),
        };

        let matched_category = apply_logcheck_rules(&log_entry);
        assert_eq!(matched_category, Some(RuleCategory::Violations));
    }

    #[test]
    fn test_system_event_matching() {
        let log_entry = LogEntry {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            message: "Started Session 123 of user alice.".to_string(),
            priority: Some("6".to_string()),
            systemd_unit: Some("systemd-logind.service".to_string()),
            pid: Some("1".to_string()),
            hostname: Some("testhost".to_string()),
            syslog_identifier: Some("systemd-logind".to_string()),
        };

        let matched_category = apply_logcheck_rules(&log_entry);
        assert_eq!(matched_category, Some(RuleCategory::SystemEvents));
    }

    #[test]
    fn test_no_match() {
        let log_entry = LogEntry {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            message: "Some random application message".to_string(),
            priority: Some("6".to_string()),
            systemd_unit: Some("myapp.service".to_string()),
            pid: Some("9999".to_string()),
            hostname: Some("testhost".to_string()),
            syslog_identifier: Some("myapp".to_string()),
        };

        let matched_category = apply_logcheck_rules(&log_entry);
        assert_eq!(matched_category, None);
    }

    #[test]
    fn test_rule_database_stats() {
        let stats = LOGCHECK_DB.get_stats();

        // Should have some rules loaded
        assert!(stats["violations_rules"] > 0);
        assert!(stats["server"] > 0);
        assert!(stats["total_rules"] > 0);

        println!("Rule database stats: {:?}", stats);
    }
}
