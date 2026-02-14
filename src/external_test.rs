use crate::rules::{LogcheckDatabase, RuleCategory, RuleError};

/// A standalone utility to test logcheck rule loading and matching
/// This function is not exported to WASM, only used for testing
#[cfg(test)]
pub fn load_external_rules_demo() -> Result<LogcheckDatabase, RuleError> {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    // Create temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create violations.d directory with security rules
    let violations_dir = base_path.join("violations.d");
    fs::create_dir_all(&violations_dir).unwrap();

    let mut violations_file = fs::File::create(violations_dir.join("security")).unwrap();
    writeln!(violations_file, "# Security violation rules").unwrap();
    writeln!(violations_file, "^.*[Aa]uthentication failure.*$").unwrap();
    writeln!(violations_file, "^.*[Ff]ailed password for.*$").unwrap();
    writeln!(violations_file, "^.*sudo.*authentication failure.*$").unwrap();

    // Create ignore.d.server directory with real systemd rules
    let server_dir = base_path.join("ignore.d.server");
    fs::create_dir_all(&server_dir).unwrap();

    // Load the downloaded systemd rules
    if let Ok(systemd_rules) = fs::read_to_string("sample-systemd-rules.txt") {
        let mut systemd_file = fs::File::create(server_dir.join("systemd")).unwrap();
        write!(systemd_file, "{}", systemd_rules).unwrap();
    }

    // Load the database
    LogcheckDatabase::load_from_directory(base_path)
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_external_rules_loading() {
        if let Ok(database) = load_external_rules_demo() {
            let stats = database.get_stats();
            println!("Loaded external rules: {:?}", stats);

            // Test with realistic systemd message
            let systemd_msg =
                "Jan 12 14:30:22 server systemd[1]: Started Session 123 of user alice.";
            let result = database.match_message(systemd_msg);
            println!("SystemD message result: {:?}", result);

            // Test with security violation
            let security_msg =
                "Jan 12 14:30:22 server sshd[1234]: Failed password for root from 192.168.1.100";
            let result = database.match_message(security_msg);
            println!("Security message result: {:?}", result);
            assert_eq!(result, Some(RuleCategory::Violations));
        } else {
            println!("External rules file not found, skipping test");
        }
    }
}
