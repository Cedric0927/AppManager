fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_index = 0usize;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }
    let digits = if unit_index == 0 { 0 } else if value >= 10.0 { 1 } else { 2 };
    format!("{:.*} {}", digits, value, UNITS[unit_index])
}

fn main() {
    let mut apps = appmanager_lib::apps::scan_apps();
    apps.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));

    println!("apps: {}", apps.len());
    for a in apps.iter().take(30) {
        println!(
            "{:<10}  {:<40}  {:<28}  {}",
            format_bytes(a.total_bytes),
            a.name,
            a.publisher.clone().unwrap_or_default(),
            a.id
        );
    }
}
