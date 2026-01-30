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

pub fn parse_printer_option(s: &str) -> Result<PrinterOption, String> {
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
