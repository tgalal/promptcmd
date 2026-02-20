use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::OnceLock;
use tokio::sync::oneshot::Receiver;
use tokio::time::{sleep};
use std::time::Duration;
use openssh::Stdio;
use thiserror::Error;
use std::thread;

#[derive(Error, Debug)]
pub enum ParsedSshArgsError {
    #[error("{0}")]
    Other(String),
}

/// SSH option information - maps option character to its parameter description
/// Empty string means it's a boolean flag (no parameter)
type SshOptions = HashMap<String, String>;

/// Cached SSH options discovered from the system's SSH binary
static SSH_OPTIONS: OnceLock<SshOptions> = OnceLock::new();

fn check_master_ready(
    control_path: &str,
    host: &str,
    port: u32,
    ) -> Result<bool, String> {

    let args = ["-p", &port.to_string(), "-O", "check", "-o", &format!("ControlPath={}", control_path), host];

    let status = Command::new("ssh")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if let Ok(status) = status {
        if status.success() {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn wait_for_master_ready(
    control_path: &str,
    host: &str,
    port: u32,
    timeout: Duration) -> Result<(), String> {
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if check_master_ready(control_path, host, port)? {
            return Ok(())
        }
        thread::sleep(Duration::from_millis(200));
    }

    Err("Timeout: master connection failed to establish".to_string())
}

pub enum WaitForMasterResult {
    Established,
    Aborted,
    Timeout
}

pub async fn async_wait_for_master_ready(
    control_path: &str,
    host: &str,
    port: u32,
    timeout: Duration,
    mut abort_rx: Receiver<()>
) -> Result<WaitForMasterResult, String> {
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if check_master_ready(control_path, host, port)? {
            return Ok(WaitForMasterResult::Established)
        }

        tokio::select! {
            _ = sleep(Duration::from_millis(200)) => {
            }
            _ = &mut abort_rx => {
                return Ok(WaitForMasterResult::Aborted)
            }
        }
    }

    Ok(WaitForMasterResult::Timeout)
}


/// Get SSH options by parsing ssh's help output or using fallback
fn ssh_options() -> &'static SshOptions {
    SSH_OPTIONS.get_or_init(|| {
        // Try to get options from SSH help output
        if let Ok(opts) = discover_ssh_options() {
            if !opts.is_empty() {
                return opts;
            }
        }

        // Fallback to hardcoded options (covers most common SSH versions)
        fallback_ssh_options()
    })
}

/// Discover SSH options by parsing the ssh command's stderr
fn discover_ssh_options() -> Result<SshOptions, Box<dyn std::error::Error>> {
    let output = Command::new("ssh")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check for OpenSSL version mismatch which pollutes stderr
    if stderr.is_empty() || stderr.contains("OpenSSL version mismatch.") {
        return Ok(HashMap::new());
    }

    let mut options = HashMap::new();
    let text = stderr.as_ref();
    let mut pos = 0;

    // Parse bracketed option groups like [-46AaCfGgKkMNnqsTtVvXxYy] and [-b bind_address]
    while let Some(start) = text[pos..].find('[') {
        let start = pos + start;

        // Find matching closing bracket
        let mut bracket_count = 1;
        let mut end = start;

        while bracket_count > 0 {
            end += 1;
            if end >= text.len() {
                break;
            }
            match text.as_bytes()[end] {
                b'[' => bracket_count += 1,
                b']' => bracket_count -= 1,
                _ => {}
            }
        }

        if bracket_count != 0 {
            break; // Unmatched brackets
        }

        let content = &text[start + 1..end];
        pos = end;

        if content.len() < 2 || !content.starts_with('-') {
            continue;
        }

        // Check if this is an option with description like "-b bind_address"
        if let Some(space_pos) = content.find(' ') {
            let opt = &content[1..space_pos]; // Skip the '-'
            let desc = &content[space_pos + 1..];
            options.insert(opt.to_string(), desc.to_string());
        } else {
            // Multiple boolean flags like "-46AaCfGgKk"
            for ch in content[1..].chars() {
                options.insert(ch.to_string(), String::new());
            }
        }
    }

    Ok(options)
}

/// Fallback SSH options for common OpenSSH versions
fn fallback_ssh_options() -> SshOptions {
    let mut opts = HashMap::new();

    // Boolean flags (no argument)
    for flag in ["4", "6", "A", "a", "C", "f", "G", "g", "K", "k",
                "M", "N", "n", "q", "s", "T", "t", "V", "v", "X",
                "x", "Y", "y"] {
        opts.insert(flag.to_string(), String::new());
    }

    // Flags that take arguments
    let with_args = [
        ("B", "bind_interface"),
        ("b", "bind_address"),
        ("c", "cipher_spec"),
        ("D", "[bind_address:]port"),
        ("E", "log_file"),
        ("e", "escape_char"),
        ("F", "configfile"),
        ("I", "pkcs11"),
        ("i", "identity_file"),
        ("J", "[user@]host[:port]"),
        ("L", "address"),
        ("l", "login_name"),
        ("m", "mac_spec"),
        ("O", "ctl_cmd"),
        ("o", "option"),
        ("p", "port"),
        ("Q", "query_option"),
        ("R", "address"),
        ("S", "ctl_path"),
        ("W", "host:port"),
        ("w", "local_tun[:remote_tun]"),
    ];

    for (flag, desc) in with_args {
        opts.insert(flag.to_string(), desc.to_string());
    }

    opts
}

/// Get sets of boolean and other SSH arguments
fn get_ssh_cli() -> (HashSet<String>, HashSet<String>) {
    let mut boolean_args = HashSet::new();
    let mut other_args = HashSet::new();

    for (key, val) in ssh_options() {
        let arg = format!("-{}", key);
        if val.is_empty() {
            boolean_args.insert(arg);
        } else {
            other_args.insert(arg);
        }
    }

    (boolean_args, other_args)
}

/// Destination information extracted from SSH arguments
#[derive(Debug, Clone)]
pub struct SshDestination {
    pub username: Option<String>,
    pub hostname: String,
    pub hostname_for_match: String,
}

/// Parse result containing SSH args, server args, and whether to passthrough
#[derive(Debug)]
pub struct ParsedSshArgs {
    pub ssh_args: Vec<String>,
    pub server_args: Vec<String>,
    pub passthrough: bool,
}

impl ParsedSshArgs {
    pub fn port(&self) -> u32 {
        self.ssh_args.iter()
        .position(|arg| arg == "-p")
        .and_then(|i| self.ssh_args.get(i + 1))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(22)
    }
}

/// Parse SSH command line arguments
pub fn parse_ssh_args(args: &[String]) -> Result<ParsedSshArgs, ParsedSshArgsError> {
    if args.is_empty() {
        return Ok(ParsedSshArgs {
            ssh_args: vec![],
            server_args: vec![],
            passthrough: true,
        });
    }

    let passthrough_args: HashSet<&str> =
        ["-N", "-n", "-f", "-G", "-T", "-V"].iter().copied().collect();

    let (boolean_ssh_args, other_ssh_args) = get_ssh_cli();

    let mut ssh_args = Vec::new();
    let mut server_args = Vec::new();
    let mut passthrough = false;
    let mut expecting_option_val = false;
    let mut stop_option_processing = false;

    for argument in args {
        // Once we have a hostname (server_args non-empty) or hit --,
        // everything else is a server arg
        if !server_args.is_empty() || stop_option_processing {
            server_args.push(argument.clone());
            continue;
        }

        if argument.starts_with('-') && !expecting_option_val {
            if argument == "--" {
                stop_option_processing = true;
                continue;
            }

            // Handle multi-character options like -vvv or -p22
            let all_chars: Vec<char> = argument.chars().skip(1).collect();

            let mut i = 0;
            while i < all_chars.len() {
                let ch = all_chars[i];
                let arg = format!("-{}", ch);

                if passthrough_args.contains(arg.as_str()) {
                    passthrough = true;
                }

                if boolean_ssh_args.contains(&arg) {
                    ssh_args.push(arg);
                    i += 1;
                    continue;
                }

                if other_ssh_args.contains(&arg) {
                    ssh_args.push(arg);

                    // Check if value is attached (like -p22)
                    if i + 1 < all_chars.len() {
                        let rest: String = all_chars[i + 1..].iter().collect();
                        ssh_args.push(rest);
                        break;
                    } else {
                        expecting_option_val = true;
                        break;
                    }
                }

                return Err(ParsedSshArgsError::Other(format!("unknown option -- {}", ch)));
            }
            continue;
        }

        if expecting_option_val {
            ssh_args.push(argument.clone());
            expecting_option_val = false;
            continue;
        }

        // This is the hostname/destination
        server_args.push(argument.clone());
    }

    if server_args.is_empty() && !passthrough {
        return Err(ParsedSshArgsError::Other("No hostname specified".to_string()));
    }

    Ok(ParsedSshArgs {
        ssh_args,
        server_args,
        passthrough,
    })
}

/// Extract destination (username and hostname) from SSH arguments
pub fn get_destination(hostname: &str) -> SshDestination {
    let mut username = None;
    let mut hostname_for_match = hostname.to_string();

    // Try to get current username as default
    if let Ok(user) = std::env::var("USER") {
        username = Some(user);
    }

    // Parse ssh:// URLs
    if hostname.starts_with("ssh://") {
        if let Ok(url) = url::Url::parse(hostname) {
            if let Some(host) = url.host_str() {
                hostname_for_match = host.to_string();
            }
            if !url.username().is_empty() {
                username = Some(url.username().to_string());
            }
        }
    } else if hostname.contains('@') && !hostname.starts_with('@') {
        // Parse user@host format
        if let Some((user, host)) = hostname.split_once('@') {
            username = Some(user.to_string());
            hostname_for_match = host.to_string();
        }
    }

    SshDestination {
        username,
        hostname: hostname.to_string(),
        hostname_for_match,
    }
}

/// Main entry point: parse full SSH command line and extract destination
pub fn parse_ssh_command(args: &[String]) -> Result<(SshDestination, ParsedSshArgs), ParsedSshArgsError> {
    let parsed = parse_ssh_args(args)?;

    if parsed.server_args.is_empty() {
        return Err(ParsedSshArgsError::Other("No hostname found in arguments".to_string()));
    }

    let hostname = &parsed.server_args[0];
    let destination = get_destination(hostname);

    Ok((destination, parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let args = vec!["user@example.com".to_string()];
        let (dest, parsed) = parse_ssh_command(&args).unwrap();

        assert_eq!(dest.hostname_for_match, "example.com");
        assert_eq!(dest.username, Some("user".to_string()));
        assert_eq!(parsed.server_args, vec!["user@example.com"]);
    }

    #[test]
    fn test_parse_with_flags() {
        let args = vec![
            "-p".to_string(),
            "2222".to_string(),
            "-v".to_string(),
            "user@host".to_string(),
            "echo".to_string(),
            "hello".to_string(),
        ];

        let (dest, parsed) = parse_ssh_command(&args).unwrap();

        assert_eq!(dest.hostname_for_match, "host");
        assert_eq!(dest.username, Some("user".to_string()));
        assert_eq!(parsed.ssh_args, vec!["-p", "2222", "-v"]);
        assert_eq!(parsed.server_args, vec!["user@host", "echo", "hello"]);
    }

    #[test]
    fn test_parse_combined_flags() {
        let args = vec!["-vvv".to_string(), "host".to_string()];
        let (dest, parsed) = parse_ssh_command(&args).unwrap();

        assert_eq!(dest.hostname_for_match, "host");
        assert_eq!(parsed.ssh_args, vec!["-v", "-v", "-v"]);
    }

    #[test]
    fn test_parse_attached_value() {
        let args = vec!["-p22".to_string(), "host".to_string()];
        let (dest, parsed) = parse_ssh_command(&args).unwrap();

        assert_eq!(dest.hostname_for_match, "host");
        assert_eq!(parsed.ssh_args, vec!["-p", "22"]);
    }

    #[test]
    fn test_parse_ssh_url() {
        let args = vec!["ssh://user@example.com:2222".to_string()];
        let (dest, _) = parse_ssh_command(&args).unwrap();

        assert_eq!(dest.hostname_for_match, "example.com");
        assert_eq!(dest.username, Some("user".to_string()));
    }

    #[test]
    fn test_parse_with_separator() {
        let args = vec![
            "-p".to_string(),
            "22".to_string(),
            "--".to_string(),
            "host".to_string(),
            "ls".to_string(),
            "-la".to_string(),
        ];

        let (dest, parsed) = parse_ssh_command(&args).unwrap();

        assert_eq!(dest.hostname_for_match, "host");
        assert_eq!(parsed.ssh_args, vec!["-p", "22"]);
        assert_eq!(parsed.server_args, vec!["host", "ls", "-la"]);
    }
}

