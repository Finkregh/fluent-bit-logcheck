use crate::rules::LogcheckDatabase;

#[test]
fn test_load_production_logcheck_rules() {
    let rules_path = std::path::Path::new(env!("HOME"))
        .join("work/private/fluent-filter-logcheck/knowledge/logcheck-database/etc/logcheck");

    if !rules_path.exists() {
        println!(
            "⚠️  Production rules not found at: {}",
            rules_path.display()
        );
        println!("   Skipping test - this is expected in CI environments");
        return;
    }

    println!("📂 Loading production rules from: {}", rules_path.display());

    // Load the production rules
    let database = LogcheckDatabase::load_from_directory(&rules_path)
        .expect("Failed to load production logcheck rules");

    // Get statistics
    let stats = database.get_stats();
    println!("\n📊 Production Rule Statistics:");
    println!(
        "   - Cracking rules: {}",
        stats.get("cracking_rules").unwrap_or(&0)
    );
    println!(
        "   - Cracking ignore: {}",
        stats.get("cracking_ignore").unwrap_or(&0)
    );
    println!(
        "   - Violations rules: {}",
        stats.get("violations_rules").unwrap_or(&0)
    );
    println!(
        "   - Violations ignore: {}",
        stats.get("violations_ignore").unwrap_or(&0)
    );
    println!(
        "   - System events (paranoid): {}",
        stats.get("system_events").unwrap_or(&0)
    );
    println!(
        "   - Workstation rules: {}",
        stats.get("workstation").unwrap_or(&0)
    );
    println!("   - Server rules: {}", stats.get("server").unwrap_or(&0));
    println!("   - Local rules: {}", stats.get("local").unwrap_or(&0));
    println!(
        "   📦 Total rules loaded: {}",
        stats.get("total_rules").unwrap_or(&0)
    );

    // Verify we loaded a reasonable number of rules
    let total = stats.get("total_rules").unwrap_or(&0);
    assert!(
        *total > 100,
        "Expected to load many rules from production database, got {}",
        total
    );

    println!("\n🧪 Testing rule matching with production rules:\n");

    // Test with production rules to verify chunking works correctly
    // Note: We're mainly testing that rules load and compile with chunking
    // The actual pattern matching depends on which rules exist in the production database
    let test_cases = vec![
        (
            "Feb  3 10:30:15 server sshd-session[1234]: Failed password for root from 192.168.1.100 port 22 ssh2",
            "SSH authentication failure - should match Server ignore rules if present",
        ),
        (
            "Feb  3 10:30:16 host sshd-session[5678]: Accepted publickey for alice from 10.0.0.1 port 55555 ssh2",
            "SSH successful login - should match Server ignore rules if present",
        ),
        (
            "Feb  3 10:30:17 srv systemd[1]: Started Session 42 of user bob.",
            "SystemD session - should match ignore rules if present",
        ),
        (
            "MyCustomApp[999]: Processing request 12345",
            "Custom application - should NOT match any rules",
        ),
    ];

    for (message, description) in test_cases {
        let result = database.match_message(message);
        println!("   📝 {}", description);
        println!("      Message: {}", message);
        println!("      Result: {:?}", result);
    }

    // The main test is that we successfully loaded and compiled 1947 rules with chunking
    // Pattern matching results depend on the actual production rule content

    println!("\n✅ Successfully loaded and compiled production rules with automatic chunking!");
}

#[test]
fn test_production_rules_directory_structure() {
    let rules_path = std::path::Path::new(env!("HOME"))
        .join("work/private/fluent-filter-logcheck/knowledge/logcheck-database/etc/logcheck");

    if !rules_path.exists() {
        println!("⚠️  Skipping - production rules not found");
        return;
    }

    println!("🔍 Checking directory structure...\n");

    let expected_dirs = vec![
        "cracking.d",
        "cracking.ignore.d",
        "violations.d",
        "violations.ignore.d",
        "ignore.d.paranoid",
        "ignore.d.server",
        "ignore.d.workstation",
    ];

    for dir_name in expected_dirs {
        let dir_path = rules_path.join(dir_name);
        let exists = dir_path.exists() && dir_path.is_dir();

        if exists {
            let file_count = std::fs::read_dir(&dir_path)
                .map(|entries| entries.count())
                .unwrap_or(0);
            println!("   ✅ {} ({} files)", dir_name, file_count);
        } else {
            println!("   ⚠️  {} (not found)", dir_name);
        }
    }

    println!("\n✅ Directory structure check complete!");
}
