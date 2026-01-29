use clap::Parser;

/// Configure CUPS printers and classes
#[derive(Parser, Debug)]
#[command(name = "lpadmin-rs")]
#[command(version)]
#[command(disable_help_flag = true)]
struct Args {
    /// Print help information
    #[arg(long = "help", action = clap::ArgAction::Help)]
    help: (),

    /// Set the named destination as the server default
    #[arg(short = 'd', value_name = "DESTINATION")]
    default_printer: Option<String>,

    /// Specify/add the named destination
    #[arg(short = 'p', value_name = "DESTINATION")]
    printer: Option<String>,

    /// Remove the named destination
    #[arg(short = 'x', value_name = "DESTINATION")]
    delete_printer: Option<String>,

    /// Add the named destination to a class
    #[arg(short = 'c', value_name = "CLASS")]
    class: Option<String>,

    /// Specify the textual description of the printer
    #[arg(short = 'D', value_name = "DESCRIPTION")]
    description: Option<String>,

    /// Encrypt the connection to the server (before -p)
    #[arg(
        short = 'E',
        help = "Encrypt the connection to the server (before -p)\nEnable and accept jobs on the printer (after -p)"
    )]
    enable_encrypt: bool,

    /// Connect to the named server and port
    #[arg(short = 'h', value_name = "SERVER[:PORT]")]
    server: Option<String>,

    /// Specify the textual location of the printer
    #[arg(short = 'L', value_name = "LOCATION")]
    location: Option<String>,

    /// Specify a standard model/PPD file for the printer
    #[arg(
        short = 'm',
        value_name = "MODEL",
        help = "Specify a standard model/PPD file for the printer\nUse \"everywhere\" for IPP Everywhere compatible printers"
    )]
    model: Option<String>,

    /// Set printer options (repeatable)
    #[arg(short = 'o', value_name = "NAME=VALUE", value_parser = parse_printer_option)]
    options: Vec<PrinterOption>,

    /// Remove the named destination from a class
    #[arg(short = 'r', value_name = "CLASS")]
    remove_from_class: Option<String>,

    /// Remove the default value for the named option
    #[arg(short = 'R', value_name = "NAME-DEFAULT")]
    remove_option: Option<String>,

    /// Set user-level access control
    #[arg(short = 'u', value_name = "allow:|deny:...")]
    #[arg(value_parser = parse_access_control)]
    access_control: Vec<AccessControl>,

    /// Specify the username to use for authentication
    #[arg(short = 'U', value_name = "USERNAME")]
    username: Option<String>,

    /// Specify the device URI for the printer
    #[arg(short = 'v', value_name = "DEVICE-URI")]
    device_uri: Option<String>,
}

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

fn parse_access_control(s: &str) -> Result<AccessControl, String> {
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

fn parse_principals(s: &str) -> Result<Vec<Principal>, String> {
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

/// Printer options
#[derive(Debug, Clone)]
pub enum PrinterOption {
    CupsIPPSupplies(bool),
    CupsSNMPSupplies(bool),
    PrinterIsShared(bool),
    JobKLimit(u32),
    JobPageLimit(u32),
    JobQuotaPeriod(u32),
    JobSheetsDefault(String),
    PortMonitor(String),
    PrinterErrorPolicy(String),
    PrinterOpPolicy(String),
    Other { key: String, value: String },
}

#[derive(Debug, Default)]
pub struct PrinterOptions {
    pub cups_ipp_supplies: Option<bool>,
    pub cups_snmp_supplies: Option<bool>,
    pub printer_is_shared: Option<bool>,
    pub job_k_limit: Option<u32>,
    pub job_page_limit: Option<u32>,
    pub job_quota_period: Option<u32>,
    pub job_sheets_default: Option<String>,
    pub port_monitor: Option<String>,
    pub printer_error_policy: Option<String>,
    pub printer_op_policy: Option<String>,
    /// PPD and default options
    pub other: Vec<(String, String)>,
}

impl From<Vec<PrinterOption>> for PrinterOptions {
    fn from(opts: Vec<PrinterOption>) -> Self {
        opts.into_iter().fold(Self::default(), |mut acc, opt| {
            match opt {
                PrinterOption::CupsIPPSupplies(v) => acc.cups_ipp_supplies = Some(v),
                PrinterOption::CupsSNMPSupplies(v) => acc.cups_snmp_supplies = Some(v),
                PrinterOption::PrinterIsShared(v) => acc.printer_is_shared = Some(v),
                PrinterOption::JobKLimit(v) => acc.job_k_limit = Some(v),
                PrinterOption::JobPageLimit(v) => acc.job_page_limit = Some(v),
                PrinterOption::JobQuotaPeriod(v) => acc.job_quota_period = Some(v),
                PrinterOption::JobSheetsDefault(v) => acc.job_sheets_default = Some(v),
                PrinterOption::PortMonitor(v) => acc.port_monitor = Some(v),
                PrinterOption::PrinterErrorPolicy(v) => acc.printer_error_policy = Some(v),
                PrinterOption::PrinterOpPolicy(v) => acc.printer_op_policy = Some(v),
                PrinterOption::Other { key, value } => acc.other.push((key, value)),
            }
            acc
        })
    }
}

fn parse_printer_option(s: &str) -> Result<PrinterOption, String> {
    let (key, val) = s
        .split_once('=')
        .ok_or_else(|| format!("expected NAME=VALUE, got: {s}"))?;

    match key {
        "cupsIPPSupplies" => Ok(PrinterOption::CupsIPPSupplies(parse_bool(val)?)),
        "cupsSNMPSupplies" => Ok(PrinterOption::CupsSNMPSupplies(parse_bool(val)?)),
        "printer-is-shared" => Ok(PrinterOption::PrinterIsShared(parse_bool(val)?)),
        "job-k-limit" => Ok(PrinterOption::JobKLimit(parse_u32(val)?)),
        "job-page-limit" => Ok(PrinterOption::JobPageLimit(parse_u32(val)?)),
        "job-quota-period" => Ok(PrinterOption::JobQuotaPeriod(parse_u32(val)?)),
        "job-sheets-default" => Ok(PrinterOption::JobSheetsDefault(val.to_string())),
        "port-monitor" => Ok(PrinterOption::PortMonitor(val.to_string())),
        "printer-error-policy" => Ok(PrinterOption::PrinterErrorPolicy(val.to_string())),
        "printer-op-policy" => Ok(PrinterOption::PrinterOpPolicy(val.to_string())),
        _ => Ok(PrinterOption::Other {
            key: key.to_string(),
            value: val.to_string(),
        }),
    }
}

fn parse_bool(s: &str) -> Result<bool, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("expected true/false/yes/no/on/off, got: {s}")),
    }
}

fn parse_u32(s: &str) -> Result<u32, String> {
    s.trim()
        .parse()
        .map_err(|_| format!("expected integer, got: {s}"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let opts: PrinterOptions = args.options.into();

    // FIXME: remove temporary dbg! for debugging
    dbg!(&args.printer);
    dbg!(&args.default_printer);
    dbg!(&args.delete_printer);
    dbg!(&args.class);
    dbg!(&args.remove_from_class);
    dbg!(&args.device_uri);
    dbg!(&args.model);
    dbg!(&args.description);
    dbg!(&args.location);
    dbg!(&args.server);
    dbg!(&args.username);
    dbg!(&args.enable_encrypt);
    dbg!(&args.access_control);
    dbg!(&opts);

    Ok(())
}
