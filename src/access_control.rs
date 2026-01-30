/// User-level access control for a printer
#[derive(Debug, Clone)]
pub enum AccessControl {
    AllowAll,
    DenyNone,
    Allow(Vec<Principal>),
    Deny(Vec<Principal>),
}

/// A user or group principal for access control
#[derive(Debug, Clone)]
pub enum Principal {
    User(String),
    Group(String),
}

pub fn parse_access_control(s: &str) -> Result<AccessControl, String> {
    if let Some(rest) = s.strip_prefix("allow:") {
        if rest == "all" {
            Ok(AccessControl::AllowAll)
        } else {
            Ok(AccessControl::Allow(parse_principals(rest)?))
        }
    } else if let Some(rest) = s.strip_prefix("deny:") {
        if rest == "none" {
            Ok(AccessControl::DenyNone)
        } else {
            Ok(AccessControl::Deny(parse_principals(rest)?))
        }
    } else {
        Err(format!("expected allow:... or deny:..., got: {s}"))
    }
}

pub fn parse_principals(s: &str) -> Result<Vec<Principal>, String> {
    s.split(',')
        .map(|p| {
            let p = p.trim();
            if p.is_empty() {
                Err("empty principal".to_string())
            } else if let Some(group) = p.strip_prefix('@') {
                Ok(Principal::Group(group.to_string()))
            } else {
                Ok(Principal::User(p.to_string()))
            }
        })
        .collect()
}
