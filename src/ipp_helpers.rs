use crate::access_control::{AccessControl, Principal};
use crate::options::PrinterOptions;
use cups_rs::bindings::{
    _ipp_s, cups_ptype_t, cupsLastErrorString, ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_CLASS,
    ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_PRINTER, ipp_op_e_IPP_OP_CUPS_DELETE_CLASS,
    ipp_op_e_IPP_OP_CUPS_DELETE_PRINTER, ipp_op_e_IPP_OP_CUPS_SET_DEFAULT,
    ipp_op_e_IPP_OP_ENABLE_PRINTER, ipp_op_e_IPP_OP_GET_PRINTER_ATTRIBUTES,
    ipp_op_e_IPP_OP_RESUME_PRINTER, ipp_status_e_IPP_STATUS_OK,
    ipp_status_e_IPP_STATUS_OK_CONFLICTING, ipp_status_e_IPP_STATUS_OK_IGNORED_OR_SUBSTITUTED,
    ipp_tag_e_IPP_TAG_CHARSET, ipp_tag_e_IPP_TAG_DELETEATTR, ipp_tag_e_IPP_TAG_ENUM,
    ipp_tag_e_IPP_TAG_INTEGER, ipp_tag_e_IPP_TAG_KEYWORD, ipp_tag_e_IPP_TAG_LANGUAGE,
    ipp_tag_e_IPP_TAG_NAME, ipp_tag_e_IPP_TAG_OPERATION, ipp_tag_e_IPP_TAG_PRINTER,
    ipp_tag_e_IPP_TAG_TEXT, ipp_tag_e_IPP_TAG_URI, ippAddBoolean, ippAddInteger, ippAddString,
    ippAddStrings, ippDelete, ippFindAttribute, ippGetCount, ippGetInteger, ippGetStatusCode,
    ippGetString, ippNewRequest,
};
use cups_rs::connection::HttpConnection;
use cups_rs::constants::PRINTER_CLASS;
use cups_rs::{
    ConnectionFlags, IppOperation, IppRequest, IppTag, IppValueTag, get_all_destinations,
    get_default_destination,
};
use std::ffi::{CStr, CString};
use std::ptr;

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

    let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_PRINTER as i32) };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }

    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", &printer_uri)?;

    if let Some(uri) = device_uri {
        add_printer_uri_attr(request, "device-uri", uri)?;
    }
    if let Some(m) = model {
        add_printer_name_attr(request, "ppd-name", m)?;
    }
    if let Some(d) = description {
        add_printer_text_attr(request, "printer-info", d)?;
    }
    if let Some(l) = location {
        add_printer_text_attr(request, "printer-location", l)?;
    }

    apply_printer_options(request, opts)?;
    apply_access_control(request, access)?;

    let response = send_ipp_request(request, &connection, "/admin/", "CUPS-ADD-MODIFY-PRINTER")?;
    unsafe { ippDelete(response) };
    Ok(())
}

/// Set the default destination (CUPS-SET-DEFAULT)
pub fn set_default_printer(printer: &str) -> Result<(), Box<dyn std::error::Error>> {
    let connection = scheduler_connection()?;
    let printer_uri = format!("ipp://localhost/printers/{printer}");

    let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_CUPS_SET_DEFAULT as i32) };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }

    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", &printer_uri)?;

    let response = send_ipp_request(request, &connection, "/admin/", "CUPS-SET-DEFAULT")?;
    unsafe { ippDelete(response) };
    Ok(())
}

/// Delete a printer (CUPS-DELETE-PRINTER)
pub fn delete_printer(printer: &str) -> Result<(), Box<dyn std::error::Error>> {
    let connection = scheduler_connection()?;
    let printer_uri = format!("ipp://localhost/printers/{printer}");

    let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_CUPS_DELETE_PRINTER as i32) };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }

    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", &printer_uri)?;

    let response = send_ipp_request(request, &connection, "/admin/", "CUPS-DELETE-PRINTER")?;
    unsafe { ippDelete(response) };
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

    let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_CLASS as i32) };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }

    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", &class_uri)?;

    let name = CString::new("member-uris")?;
    let values: Vec<CString> = existing_members
        .iter()
        .map(|v| CString::new(v.as_str()))
        .collect::<Result<_, _>>()?;
    let value_ptrs: Vec<*const i8> = values.iter().map(|v| v.as_ptr()).collect();

    unsafe {
        ippAddStrings(
            request,
            ipp_tag_e_IPP_TAG_PRINTER,
            ipp_tag_e_IPP_TAG_URI,
            name.as_ptr(),
            value_ptrs.len() as i32,
            ptr::null(),
            value_ptrs.as_ptr(),
        );
    }

    let response = send_ipp_request(request, &connection, "/admin/", "CUPS-ADD-MODIFY-CLASS")?;
    unsafe { ippDelete(response) };

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
            "lpadmin-rs: GET-PRINTER-ATTRIBUTES failed: {}",
            cups_last_error()
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
    let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_ENABLE_PRINTER as i32) };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }
    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", &printer_uri)?;
    let response = send_ipp_request(request, &connection, "/admin/", "IPP-ENABLE-PRINTER")?;
    unsafe { ippDelete(response) };

    // Resume
    let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_RESUME_PRINTER as i32) };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }
    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", &printer_uri)?;
    let response = send_ipp_request(request, &connection, "/admin/", "IPP-RESUME-PRINTER")?;
    unsafe { ippDelete(response) };

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
        let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_CUPS_DELETE_CLASS as i32) };
        if request.is_null() {
            return Err("Failed to create IPP request".into());
        }

        add_standard_ipp_attrs(request)?;
        add_string_attr(request, "printer-uri", &class_uri)?;

        let response = send_ipp_request(request, &connection, "/admin/", "CUPS-DELETE-CLASS")?;
        unsafe { ippDelete(response) };

        return Ok(());
    }

    let mut new_uris = Vec::new();
    for (i, uri) in member_uris.iter().enumerate() {
        if i != index {
            new_uris.push(uri.clone());
        }
    }

    let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_CLASS as i32) };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }

    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", &class_uri)?;

    let name = CString::new("member-uris")?;
    let values: Vec<CString> = new_uris
        .iter()
        .map(|v| CString::new(v.as_str()))
        .collect::<Result<_, _>>()?;
    let value_ptrs: Vec<*const i8> = values.iter().map(|v| v.as_ptr()).collect();

    unsafe {
        ippAddStrings(
            request,
            ipp_tag_e_IPP_TAG_PRINTER,
            ipp_tag_e_IPP_TAG_URI,
            name.as_ptr(),
            value_ptrs.len() as i32,
            ptr::null(),
            value_ptrs.as_ptr(),
        );
    }

    let response = send_ipp_request(request, &connection, "/admin/", "CUPS-ADD-MODIFY-CLASS")?;
    unsafe { ippDelete(response) };

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

    let request = unsafe {
        ippNewRequest(if is_class {
            ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_CLASS
        } else {
            ipp_op_e_IPP_OP_CUPS_ADD_MODIFY_PRINTER
        } as i32)
    };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }

    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", &uri)?;
    add_delete_attr(request, option)?;

    let response = send_ipp_request(request, &connection, "/admin/", "CUPS-ADD-MODIFY")?;
    unsafe { ippDelete(response) };

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
    request: *mut _ipp_s,
    connection: &HttpConnection,
    resource: &str,
    op_name: &str,
) -> Result<*mut _ipp_s, Box<dyn std::error::Error>> {
    let resource = CString::new(resource)?;
    let response = unsafe {
        cups_rs::bindings::cupsDoRequest(connection.as_ptr(), request, resource.as_ptr())
    };

    if response.is_null() {
        return Err(format!("lpadmin-rs: {}", cups_last_error()).into());
    }

    let status = unsafe { ippGetStatusCode(response) };
    if !ipp_is_success(status) {
        unsafe { ippDelete(response) };
        return Err(format!("lpadmin-rs: {} failed: {}", op_name, cups_last_error()).into());
    }

    Ok(response)
}

/// Check if response to the ipp request was a success
#[allow(non_upper_case_globals)]
fn ipp_is_success(status: i32) -> bool {
    match status {
        ipp_status_e_IPP_STATUS_OK => true,
        ipp_status_e_IPP_STATUS_OK_CONFLICTING => true,
        ipp_status_e_IPP_STATUS_OK_IGNORED_OR_SUBSTITUTED => true,
        _ => false,
    }
}

fn get_printer_type(
    connection: &HttpConnection,
    printer: &str,
    uri_out: &mut String,
) -> Result<cups_ptype_t, Box<dyn std::error::Error>> {
    *uri_out = format!("ipp://localhost/printers/{printer}");

    let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_GET_PRINTER_ATTRIBUTES as i32) };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }

    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", uri_out)?;
    add_operation_keyword_attr(request, "requested-attributes", "printer-type")?;

    let response = send_ipp_request(request, connection, "/", "GET-PRINTER-ATTRIBUTES")?;

    let mut ptype = cups_ptype_t::from(0u32);
    unsafe {
        let attr = ippFindAttribute(
            response,
            CString::new("printer-type")?.as_ptr(),
            ipp_tag_e_IPP_TAG_ENUM,
        );
        if !attr.is_null() {
            ptype = ippGetInteger(attr, 0) as cups_ptype_t;
            if (ptype & PRINTER_CLASS) != 0 {
                *uri_out = format!("ipp://localhost/classes/{printer}");
            }
        }
    }

    unsafe { ippDelete(response) };
    Ok(ptype)
}

fn get_class_members_detail(
    connection: &HttpConnection,
    class_uri: &str,
) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    let request = unsafe { ippNewRequest(ipp_op_e_IPP_OP_GET_PRINTER_ATTRIBUTES as i32) };
    if request.is_null() {
        return Err("Failed to create IPP request".into());
    }

    add_standard_ipp_attrs(request)?;
    add_string_attr(request, "printer-uri", class_uri)?;
    add_operation_keywords_attr(
        request,
        "requested-attributes",
        &["member-names", "member-uris"],
    )?;

    let response = send_ipp_request(request, connection, "/", "GET-PRINTER-ATTRIBUTES")?;

    let mut names = Vec::new();
    unsafe {
        let attr = ippFindAttribute(
            response,
            CString::new("member-names")?.as_ptr(),
            ipp_tag_e_IPP_TAG_NAME,
        );
        if attr.is_null() {
            ippDelete(response);
            return Err("lpadmin-rs: No member names were seen.".into());
        }
        let count = ippGetCount(attr);
        for i in 0..count {
            let value = ippGetString(attr, i, ptr::null_mut());
            if !value.is_null() {
                names.push(CStr::from_ptr(value).to_string_lossy().into_owned());
            }
        }
    }

    let mut uris = Vec::new();
    unsafe {
        let attr = ippFindAttribute(
            response,
            CString::new("member-uris")?.as_ptr(),
            ipp_tag_e_IPP_TAG_URI,
        );
        if !attr.is_null() {
            let count = ippGetCount(attr);
            for i in 0..count {
                let value = ippGetString(attr, i, ptr::null_mut());
                if !value.is_null() {
                    uris.push(CStr::from_ptr(value).to_string_lossy().into_owned());
                }
            }
        }
    }

    unsafe { ippDelete(response) };
    Ok((names, uris))
}

fn apply_access_control(
    request: *mut _ipp_s,
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
    request: *mut _ipp_s,
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
        add_printer_int_attr(request, "job-k-limit", v)?;
    }
    if let Some(v) = opts.job_page_limit {
        add_printer_int_attr(request, "job-page-limit", v)?;
    }
    if let Some(v) = opts.job_quota_period {
        add_printer_int_attr(request, "job-quota-period", v)?;
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
    request: *mut _ipp_s,
    name: &str,
    value: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(name)?;
    unsafe {
        ippAddBoolean(
            request,
            ipp_tag_e_IPP_TAG_PRINTER,
            name.as_ptr(),
            value as i8,
        );
    }
    Ok(())
}

fn add_standard_ipp_attrs(request: *mut _ipp_s) -> Result<(), Box<dyn std::error::Error>> {
    let charset = CString::new("utf-8")?;
    let language = CString::new("en")?;
    let user = CString::new(std::env::var("USER").unwrap_or_else(|_| "unknown".into()))?;

    let name_charset = CString::new("attributes-charset")?;
    let name_language = CString::new("attributes-natural-language")?;
    let name_user = CString::new("requesting-user-name")?;

    unsafe {
        ippAddString(
            request,
            ipp_tag_e_IPP_TAG_OPERATION,
            ipp_tag_e_IPP_TAG_CHARSET,
            name_charset.as_ptr(),
            ptr::null(),
            charset.as_ptr(),
        );
        ippAddString(
            request,
            ipp_tag_e_IPP_TAG_OPERATION,
            ipp_tag_e_IPP_TAG_LANGUAGE,
            name_language.as_ptr(),
            ptr::null(),
            language.as_ptr(),
        );
        ippAddString(
            request,
            ipp_tag_e_IPP_TAG_OPERATION,
            ipp_tag_e_IPP_TAG_NAME,
            name_user.as_ptr(),
            ptr::null(),
            user.as_ptr(),
        );
    }

    Ok(())
}

fn add_printer_int_attr(
    request: *mut _ipp_s,
    name: &str,
    value: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(name)?;
    unsafe {
        ippAddInteger(
            request,
            ipp_tag_e_IPP_TAG_PRINTER,
            ipp_tag_e_IPP_TAG_INTEGER,
            name.as_ptr(),
            value as i32,
        );
    }
    Ok(())
}

fn add_printer_text_attr(
    request: *mut _ipp_s,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(name)?;
    let value = CString::new(value)?;
    unsafe {
        ippAddString(
            request,
            ipp_tag_e_IPP_TAG_PRINTER,
            ipp_tag_e_IPP_TAG_TEXT,
            name.as_ptr(),
            ptr::null(),
            value.as_ptr(),
        );
    }
    Ok(())
}

fn add_printer_name_attr(
    request: *mut _ipp_s,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(name)?;
    let value = CString::new(value)?;
    unsafe {
        ippAddString(
            request,
            ipp_tag_e_IPP_TAG_PRINTER,
            ipp_tag_e_IPP_TAG_NAME,
            name.as_ptr(),
            ptr::null(),
            value.as_ptr(),
        );
    }
    Ok(())
}

fn add_printer_uri_attr(
    request: *mut _ipp_s,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(name)?;
    let value = CString::new(value)?;
    unsafe {
        ippAddString(
            request,
            ipp_tag_e_IPP_TAG_PRINTER,
            ipp_tag_e_IPP_TAG_URI,
            name.as_ptr(),
            ptr::null(),
            value.as_ptr(),
        );
    }
    Ok(())
}

pub fn add_string_attr(
    request: *mut _ipp_s,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(name)?;
    let value = CString::new(value)?;
    unsafe {
        ippAddString(
            request,
            ipp_tag_e_IPP_TAG_OPERATION,
            ipp_tag_e_IPP_TAG_URI,
            name.as_ptr(),
            ptr::null(),
            value.as_ptr(),
        );
    }
    Ok(())
}

fn add_delete_attr(request: *mut _ipp_s, option: &str) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(option)?;
    unsafe {
        ippAddInteger(
            request,
            ipp_tag_e_IPP_TAG_PRINTER,
            ipp_tag_e_IPP_TAG_DELETEATTR,
            name.as_ptr(),
            0,
        );
    }
    Ok(())
}

fn add_operation_keyword_attr(
    request: *mut _ipp_s,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(name)?;
    let value = CString::new(value)?;
    unsafe {
        ippAddString(
            request,
            ipp_tag_e_IPP_TAG_OPERATION,
            ipp_tag_e_IPP_TAG_KEYWORD,
            name.as_ptr(),
            ptr::null(),
            value.as_ptr(),
        );
    }
    Ok(())
}

fn add_operation_name_attr(
    request: *mut _ipp_s,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(name)?;
    let value = CString::new(value)?;
    unsafe {
        ippAddString(
            request,
            ipp_tag_e_IPP_TAG_OPERATION,
            ipp_tag_e_IPP_TAG_NAME,
            name.as_ptr(),
            ptr::null(),
            value.as_ptr(),
        );
    }
    Ok(())
}

fn add_operation_keywords_attr(
    request: *mut _ipp_s,
    name: &str,
    values: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new(name)?;
    let cvals: Vec<CString> = values
        .iter()
        .map(|v| CString::new(*v))
        .collect::<Result<_, _>>()?;
    let ptrs: Vec<*const i8> = cvals.iter().map(|v| v.as_ptr()).collect();

    unsafe {
        ippAddStrings(
            request,
            ipp_tag_e_IPP_TAG_OPERATION,
            ipp_tag_e_IPP_TAG_KEYWORD,
            name.as_ptr(),
            ptrs.len() as i32,
            ptr::null(),
            ptrs.as_ptr(),
        );
    }
    Ok(())
}

/// Returns last cups error string
fn cups_last_error() -> String {
    unsafe {
        CStr::from_ptr(cupsLastErrorString())
            .to_string_lossy()
            .into_owned()
    }
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
