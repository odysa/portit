use std::process::Command;

pub struct PortEntry {
    pub pid: u32,
    pub process_name: String,
    pub port: u16,
    pub address: String,
    pub command: String,
}

pub fn list_listening_ports() -> Vec<PortEntry> {
    let output = Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"])
        .output()
        .ok();

    let Some(output) = output else {
        return Vec::new();
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries: Vec<PortEntry> = stdout.lines().skip(1).filter_map(parse_lsof_line).collect();

    entries.sort_by_key(|e| (e.pid, e.port));
    entries.dedup_by(|a, b| a.pid == b.pid && a.port == b.port);
    entries.sort_by_key(|e| e.port);
    fetch_commands(&mut entries);
    entries
}

fn parse_lsof_line(line: &str) -> Option<PortEntry> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 10 {
        return None;
    }
    let process_name = fields[0].to_string();
    let pid: u32 = fields[1].parse().ok()?;
    let addr_port = fields[fields.len() - 2];
    let (address, port) = parse_addr_port(addr_port)?;

    Some(PortEntry {
        pid,
        process_name,
        port,
        address,
        command: String::new(),
    })
}

fn parse_addr_port(s: &str) -> Option<(String, u16)> {
    // IPv6: [::1]:3000
    if s.starts_with('[')
        && let Some(i) = s.rfind("]:")
    {
        let addr = s[1..i].to_string();
        let port = s[i + 2..].parse().ok()?;
        return Some((addr, port));
    }
    // IPv4 or wildcard: 127.0.0.1:80, *:3000
    let colon = s.rfind(':')?;
    let addr = s[..colon].to_string();
    let port = s[colon + 1..].parse().ok()?;
    Some((addr, port))
}

fn fetch_commands(entries: &mut [PortEntry]) {
    if entries.is_empty() {
        return;
    }

    let pids: Vec<String> = entries.iter().map(|e| e.pid.to_string()).collect();
    let output = Command::new("ps")
        .args(["-ww", "-p", &pids.join(","), "-o", "pid=,command="])
        .output()
        .ok();

    let Some(output) = output else { return };
    let stdout = String::from_utf8_lossy(&output.stdout);

    let lines: Vec<&str> = stdout.lines().collect();
    for entry in entries.iter_mut() {
        let pid_str = entry.pid.to_string();
        for line in &lines {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix(&pid_str)
                && rest.starts_with(char::is_whitespace)
            {
                entry.command = rest.trim_start().to_string();
                break;
            }
        }
    }
}

pub fn kill_process(pid: u32, force: bool) -> bool {
    let sig = if force { "-KILL" } else { "-TERM" };
    Command::new("kill")
        .args([sig, &pid.to_string()])
        .status()
        .is_ok_and(|s| s.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ipv4_addr_port() {
        let (addr, port) = parse_addr_port("127.0.0.1:8080").unwrap();
        assert_eq!(addr, "127.0.0.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn parse_wildcard_addr_port() {
        let (addr, port) = parse_addr_port("*:3000").unwrap();
        assert_eq!(addr, "*");
        assert_eq!(port, 3000);
    }

    #[test]
    fn parse_ipv6_addr_port() {
        let (addr, port) = parse_addr_port("[::1]:443").unwrap();
        assert_eq!(addr, "::1");
        assert_eq!(port, 443);
    }

    #[test]
    fn parse_ipv6_full_addr() {
        let (addr, port) = parse_addr_port("[::]:9090").unwrap();
        assert_eq!(addr, "::");
        assert_eq!(port, 9090);
    }

    #[test]
    fn parse_addr_port_invalid() {
        assert!(parse_addr_port("no-colon").is_none());
        assert!(parse_addr_port("127.0.0.1:notnum").is_none());
        assert!(parse_addr_port("").is_none());
    }

    #[test]
    fn parse_lsof_line_valid() {
        let line = "node       1234 user   22u  IPv4 0x1234  0t0  TCP 127.0.0.1:3000 (LISTEN)";
        let entry = parse_lsof_line(line).unwrap();
        assert_eq!(entry.process_name, "node");
        assert_eq!(entry.pid, 1234);
        assert_eq!(entry.address, "127.0.0.1");
        assert_eq!(entry.port, 3000);
        assert_eq!(entry.command, "");
    }

    #[test]
    fn parse_lsof_line_wildcard() {
        let line = "nginx      5678 root   10u  IPv4 0xabcd  0t0  TCP *:80 (LISTEN)";
        let entry = parse_lsof_line(line).unwrap();
        assert_eq!(entry.process_name, "nginx");
        assert_eq!(entry.pid, 5678);
        assert_eq!(entry.address, "*");
        assert_eq!(entry.port, 80);
    }

    #[test]
    fn parse_lsof_line_ipv6() {
        let line = "node       1234 user   22u  IPv6 0x1234  0t0  TCP [::1]:8080 (LISTEN)";
        let entry = parse_lsof_line(line).unwrap();
        assert_eq!(entry.address, "::1");
        assert_eq!(entry.port, 8080);
    }

    #[test]
    fn parse_lsof_line_too_few_fields() {
        assert!(parse_lsof_line("short line").is_none());
        assert!(parse_lsof_line("").is_none());
    }

    #[test]
    fn parse_lsof_line_bad_pid() {
        let line = "node       notpid user   22u  IPv4 0x1234  0t0  TCP *:80 (LISTEN)";
        assert!(parse_lsof_line(line).is_none());
    }
}
