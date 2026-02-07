use crate::access_control::{AccessControl, Principal};
use crate::options::PrinterOptions;
use cups_rs::bindings::{
    cups_ptype_t, ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_CLASS, ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_PRINTER,
    ipp_op_e_IPP_OP_CUPS_DELETE_CLASS, ipp_op_e_IPP_OP_CUPS_DELETE_PRINTER,
    ipp_op_e_IPP_OP_CUPS_SET_DEFAULT, ipp_op_e_IPP_OP_ENABLE_PRINTER,
};
use cups_rs::connection::HttpConnection;
use cups_rs::constants::PRINTER_CLASS;
use cups_rs::{
    ConnectionFlags, IppOperation, IppRequest, IppTag, IppValueTag, get_all_destinations,
    get_default_destination,
};

/// Create or modify a printer (CUPS-ADD-MODIFY-PRINTER)
pub fn add_or_modify_printer(
    printer: &str,
    device_uri: Option<&str>,
    model: Option<&str>,
    description: Option<&str>,
    location: Option<&str>,
    opts: &PrinterOptions,
    access: &[AccessControl],
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = scheduler_connection()?;
    let printer_uri = format!("ipp://localhost/printers/{printer}");

    let mut request = IppRequest::new_raw(ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_PRINTER as i32)?;
    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", &printer_uri)?;

    if let Some(uri) = device_uri {
        add_printer_uri_attr(&mut request, "device-uri", uri)?;
    }
    if let Some(m) = model {
        add_printer_name_attr(&mut request, "ppd-name", m)?;
    }
    if let Some(d) = description {
        add_printer_text_attr(&mut request, "printer-info", d)?;
    }
    if let Some(l) = location {
        add_printer_text_attr(&mut request, "printer-location", l)?;
    }

    apply_printer_options(&mut request, opts)?;
    apply_access_control(&mut request, access)?;

    let _response = send_ipp_request(&request, &connection, "/admin/", "CUPS-ADD-MODIFY-PRINTER")?;
    Ok(())
}

/// Set the default destination (CUPS-SET-DEFAULT)
pub fn set_default_printer(printer: &str) -> Result<(), Box<dyn std::error::Error>> {
    let connection = scheduler_connection()?;
    let printer_uri = format!("ipp://localhost/printers/{printer}");

    let mut request = IppRequest::new_raw(ipp_op_e_IPP_OP_CUPS_SET_DEFAULT as i32)?;
    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", &printer_uri)?;

    let _response = send_ipp_request(&request, &connection, "/admin/", "CUPS-SET-DEFAULT")?;
    Ok(())
}

/// Delete a printer (CUPS-DELETE-PRINTER)
pub fn delete_printer(printer: &str) -> Result<(), Box<dyn std::error::Error>> {
    let connection = scheduler_connection()?;
    let printer_uri = format!("ipp://localhost/printers/{printer}");

    let mut request = IppRequest::new_raw(ipp_op_e_IPP_OP_CUPS_DELETE_PRINTER as i32)?;
    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", &printer_uri)?;

    let _response = send_ipp_request(&request, &connection, "/admin/", "CUPS-DELETE-PRINTER")?;
    Ok(())
}

/// Add a printer to a class (CUPS-ADD-MODIFY-CLASS)
pub fn add_printer_to_class(printer: &str, class: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !validate_name(class) {
        return Err("lpadmin-rs: Class name can only contain printable characters.".into());
    }

    let connection = scheduler_connection()?;
    let class_uri = format!("ipp://localhost/classes/{class}");
    let printer_uri = format!("ipp://localhost/printers/{printer}");

    let mut existing_members: Vec<String> = Vec::new();
    if let Ok(response) = get_class_members(&connection, &class_uri) {
        existing_members = response;
    }

    if !existing_members.iter().any(|u| u == &printer_uri) {
        existing_members.push(printer_uri);
    }

    let mut request = IppRequest::new_raw(ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_CLASS as i32)?;
    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", &class_uri)?;

    add_printer_uris_attr(&mut request, "member-uris", &existing_members)?;

    let _response = send_ipp_request(&request, &connection, "/admin/", "CUPS-ADD-MODIFY-CLASS")?;
    Ok(())
}

fn get_class_members(
    connection: &HttpConnection,
    class_uri: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut request = IppRequest::new(IppOperation::GetPrinterAttributes)?;
    request.add_string(
        IppTag::Operation,
        IppValueTag::Uri,
        "printer-uri",
        class_uri,
    )?;
    request.add_string(
        IppTag::Operation,
        IppValueTag::Name,
        "requesting-user-name",
        &current_user(),
    )?;

    let response = request.send(connection, "/")?;
    if !response.is_successful() {
        return Err(format!(
            "lpadmin-rs: GET-PRINTER-ATTRIBUTES failed: {:?}",
            response.status()
        )
        .into());
    }

    let mut members = Vec::new();
    if let Some(attr) = response.find_attribute("member-uris", Some(IppTag::Printer)) {
        for i in 0..attr.count() {
            if let Some(value) = attr.get_string(i) {
                members.push(value);
            }
        }
    }

    Ok(members)
}

/// Enable and resume a printer (IPP-ENABLE-PRINTER and IPP-RESUME-PRINTER)
pub fn enable_printer(printer: &str) -> Result<(), Box<dyn std::error::Error>> {
    let connection = scheduler_connection()?;
    let printer_uri = format!("ipp://localhost/printers/{printer}");

    // Enable
    let mut request = IppRequest::new_raw(ipp_op_e_IPP_OP_ENABLE_PRINTER as i32)?;
    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", &printer_uri)?;
    let _response = send_ipp_request(&request, &connection, "/admin/", "IPP-ENABLE-PRINTER")?;

    // Resume
    let mut request = IppRequest::new(IppOperation::ResumePrinter)?;
    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", &printer_uri)?;
    let _response = send_ipp_request(&request, &connection, "/admin/", "IPP-RESUME-PRINTER")?;

    Ok(())
}

/// Remove a printer from a class (CUPS-ADD-MODIFY-CLASS)
pub fn delete_printer_from_class(
    printer: &str,
    class: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !validate_name(class) {
        return Err("lpadmin-rs: Class name can only contain printable characters.".into());
    }

    let connection = scheduler_connection()?;
    let class_uri = format!("ipp://localhost/classes/{class}");

    let (member_names, member_uris) = get_class_members_detail(&connection, &class_uri)?;

    let index = member_names
        .iter()
        .position(|name| name.eq_ignore_ascii_case(printer))
        .ok_or_else(|| {
            format!("lpadmin-rs: Printer {printer} is not a member of class {class}.")
        })?;

    if member_names.len() == 1 {
        let mut request = IppRequest::new_raw(ipp_op_e_IPP_OP_CUPS_DELETE_CLASS as i32)?;
        request.add_standard_attrs()?;
        add_operation_uri_attr(&mut request, "printer-uri", &class_uri)?;

        let _response = send_ipp_request(&request, &connection, "/admin/", "CUPS-DELETE-CLASS")?;
        return Ok(());
    }

    let mut new_uris = Vec::new();
    for (i, uri) in member_uris.iter().enumerate() {
        if i != index {
            new_uris.push(uri.clone());
        }
    }

    let mut request = IppRequest::new_raw(ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_CLASS as i32)?;
    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", &class_uri)?;

    add_printer_uris_attr(&mut request, "member-uris", &new_uris)?;

    let _response = send_ipp_request(&request, &connection, "/admin/", "CUPS-ADD-MODIFY-CLASS")?;
    Ok(())
}

/// Remove the default value for the named option
pub fn delete_printer_option(
    printer: &str,
    option: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = scheduler_connection()?;

    let mut uri = String::new();
    let is_class = get_printer_type(&connection, printer, &mut uri)? & PRINTER_CLASS != 0;

    let mut request = IppRequest::new_raw(if is_class {
        ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_CLASS
    } else {
        ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_PRINTER
    } as i32)?;

    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", &uri)?;
    add_delete_attr(&mut request, option)?;

    let _response = send_ipp_request(&request, &connection, "/admin/", "CUPS-ADD-MODIFY")?;
    Ok(())
}

/// Get a `HttpConnection` to the scheduler
fn scheduler_connection() -> Result<HttpConnection, Box<dyn std::error::Error>> {
    let dest = match get_default_destination() {
        Ok(d) => d,
        Err(_) => {
            let list = get_all_destinations()?;
            list.into_iter().next().ok_or("No printers configured")?
        }
    };

    Ok(dest.connect(ConnectionFlags::Scheduler, Some(30000), None)?)
}

fn send_ipp_request(
    request: &IppRequest,
    connection: &HttpConnection,
    resource: &str,
    op_name: &str,
) -> Result<cups_rs::IppResponse, Box<dyn std::error::Error>> {
    let response = request.send(connection, resource)?;

    if !response.is_successful() {
        return Err(format!("lpadmin-rs: {} failed: {:?}", op_name, response.status()).into());
    }

    Ok(response)
}

/// Check printer type and resolve printer/class URI
fn get_printer_type(
    connection: &HttpConnection,
    printer: &str,
    uri_out: &mut String,
) -> Result<cups_ptype_t, Box<dyn std::error::Error>> {
    *uri_out = format!("ipp://localhost/printers/{printer}");

    let mut request = IppRequest::new(IppOperation::GetPrinterAttributes)?;
    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", uri_out)?;
    add_operation_keyword_attr(&mut request, "requested-attributes", "printer-type")?;

    let response = send_ipp_request(&request, connection, "/", "GET-PRINTER-ATTRIBUTES")?;

    let mut ptype = cups_ptype_t::from(0u32);
    if let Some(attr) = response.find_attribute("printer-type", Some(IppTag::Printer)) {
        let v = attr.get_integer(0);
        ptype = v as cups_ptype_t;
        if (ptype & PRINTER_CLASS) != 0 {
            *uri_out = format!("ipp://localhost/classes/{printer}");
        }
    }

    Ok(ptype)
}

fn get_class_members_detail(
    connection: &HttpConnection,
    class_uri: &str,
) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    let mut request = IppRequest::new(IppOperation::GetPrinterAttributes)?;
    request.add_standard_attrs()?;
    add_operation_uri_attr(&mut request, "printer-uri", class_uri)?;
    add_operation_keywords_attr(
        &mut request,
        "requested-attributes",
        &["member-names", "member-uris"],
    )?;

    let response = send_ipp_request(&request, connection, "/", "GET-PRINTER-ATTRIBUTES")?;

    let mut names = Vec::new();
    if let Some(attr) = response.find_attribute("member-names", Some(IppTag::Printer)) {
        for i in 0..attr.count() {
            if let Some(v) = attr.get_string(i) {
                names.push(v);
            }
        }
    } else {
        return Err("lpadmin-rs: No member names were seen.".into());
    }

    let mut uris = Vec::new();
    if let Some(attr) = response.find_attribute("member-uris", Some(IppTag::Printer)) {
        for i in 0..attr.count() {
            if let Some(v) = attr.get_string(i) {
                uris.push(v);
            }
        }
    }

    Ok((names, uris))
}

fn apply_access_control(
    request: &mut IppRequest,
    access: &[AccessControl],
) -> Result<(), Box<dyn std::error::Error>> {
    for rule in access {
        match rule {
            AccessControl::AllowAll => {
                add_operation_name_attr(request, "requesting-user-name-allowed", "all")?;
            }
            AccessControl::DenyNone => {
                add_operation_name_attr(request, "requesting-user-name-denied", "none")?;
            }
            AccessControl::Allow(list) => {
                let value = join_principals(list);
                add_operation_name_attr(request, "requesting-user-name-allowed", &value)?;
            }
            AccessControl::Deny(list) => {
                let value = join_principals(list);
                add_operation_name_attr(request, "requesting-user-name-denied", &value)?;
            }
        }
    }
    Ok(())
}

fn join_principals(list: &[Principal]) -> String {
    list.iter()
        .map(|p| match p {
            Principal::User(u) => u.clone(),
            Principal::Group(g) => format!("@{g}"),
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn apply_printer_options(
    request: &mut IppRequest,
    opts: &PrinterOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(v) = opts.cups_ipp_supplies {
        add_printer_bool_attr(request, "cupsIPPSupplies", v)?;
    }
    if let Some(v) = opts.cups_snmp_supplies {
        add_printer_bool_attr(request, "cupsSNMPSupplies", v)?;
    }
    if let Some(v) = opts.printer_is_shared {
        add_printer_bool_attr(request, "printer-is-shared", v)?;
    }

    if let Some(v) = opts.job_k_limit {
        add_printer_int_attr(request, "job-k-limit", v as i32)?;
    }
    if let Some(v) = opts.job_page_limit {
        add_printer_int_attr(request, "job-page-limit", v as i32)?;
    }
    if let Some(v) = opts.job_quota_period {
        add_printer_int_attr(request, "job-quota-period", v as i32)?;
    }

    if let Some(v) = &opts.job_sheets_default {
        add_printer_name_attr(request, "job-sheets-default", v)?;
    }
    if let Some(v) = &opts.port_monitor {
        add_printer_name_attr(request, "port-monitor", v)?;
    }
    if let Some(v) = &opts.printer_error_policy {
        add_printer_name_attr(request, "printer-error-policy", v)?;
    }
    if let Some(v) = &opts.printer_op_policy {
        add_printer_name_attr(request, "printer-op-policy", v)?;
    }

    Ok(())
}

fn add_printer_bool_attr(
    request: &mut IppRequest,
    name: &str,
    value: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_boolean(IppTag::Printer, name, value)?;
    Ok(())
}

fn add_printer_int_attr(
    request: &mut IppRequest,
    name: &str,
    value: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_integer(IppTag::Printer, IppValueTag::Integer, name, value)?;
    Ok(())
}

fn add_printer_text_attr(
    request: &mut IppRequest,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_string(IppTag::Printer, IppValueTag::Text, name, value)?;
    Ok(())
}

fn add_printer_name_attr(
    request: &mut IppRequest,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_string(IppTag::Printer, IppValueTag::Name, name, value)?;
    Ok(())
}

fn add_printer_uri_attr(
    request: &mut IppRequest,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_string(IppTag::Printer, IppValueTag::Uri, name, value)?;
    Ok(())
}

fn add_operation_uri_attr(
    request: &mut IppRequest,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_string(IppTag::Operation, IppValueTag::Uri, name, value)?;
    Ok(())
}

fn add_delete_attr(
    request: &mut IppRequest,
    option: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_integer(IppTag::Printer, IppValueTag::DeleteAttr, option, 0)?;
    Ok(())
}

fn add_operation_keyword_attr(
    request: &mut IppRequest,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_string(IppTag::Operation, IppValueTag::Keyword, name, value)?;
    Ok(())
}

fn add_operation_name_attr(
    request: &mut IppRequest,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_string(IppTag::Operation, IppValueTag::Name, name, value)?;
    Ok(())
}

fn add_operation_keywords_attr(
    request: &mut IppRequest,
    name: &str,
    values: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    request.add_strings(IppTag::Operation, IppValueTag::Keyword, name, values)?;
    Ok(())
}

fn add_printer_uris_attr(
    request: &mut IppRequest,
    name: &str,
    values: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let vals: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
    request.add_strings(IppTag::Printer, IppValueTag::Uri, name, &vals)?;
    Ok(())
}

/// Validate destination/class name
pub fn validate_name(name: &str) -> bool {
    let mut len = 0;

    for ch in name.chars() {
        if ch == '@' {
            break;
        }
        if ch <= ' '
            || ch == '\u{7f}'
            || ch == '/'
            || ch == '\\'
            || ch == '?'
            || ch == '\''
            || ch == '"'
            || ch == '#'
        {
            return false;
        }
        len += 1;
    }

    len < 128
}

fn current_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
}
