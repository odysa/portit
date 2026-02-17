use std::process::Command;

pub struct PortEntry {
    pub pid: u32,
    pub process_name: String,
    pub port: u16,
    pub address: String,
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

    entries.sort_by_key(|e| e.port);
    entries.dedup_by(|a, b| a.pid == b.pid && a.port == b.port);
    entries
}

fn parse_lsof_line(line: &str) -> Option<PortEntry> {
    let mut fields = line.split_whitespace();
    let process_name = fields.next()?.to_string();
    let pid: u32 = fields.next()?.parse().ok()?;

    let mut rev = line.split_whitespace().rev();
    let _listen_state = rev.next()?;
    let addr_port = rev.next()?;
    let (address, port) = parse_addr_port(addr_port)?;

    Some(PortEntry {
        pid,
        process_name,
        port,
        address,
    })
}

fn parse_addr_port(s: &str) -> Option<(String, u16)> {
    // IPv6: [::1]:3000
    if s.starts_with('[') {
        if let Some(i) = s.rfind("]:") {
            let addr = s[1..i].to_string();
            let port = s[i + 2..].parse().ok()?;
            return Some((addr, port));
        }
    }
    // IPv4 or wildcard: 127.0.0.1:80, *:3000
    let colon = s.rfind(':')?;
    let addr = s[..colon].to_string();
    let port = s[colon + 1..].parse().ok()?;
    Some((addr, port))
}

pub fn kill_process(pid: u32, force: bool) -> bool {
    let sig = if force { "-KILL" } else { "-TERM" };
    Command::new("kill")
        .args([sig, &pid.to_string()])
        .status()
        .is_ok_and(|s| s.success())
}
