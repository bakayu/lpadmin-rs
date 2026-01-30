mod access_control;
mod cli;
mod ipp_helpers;
mod options;

use crate::cli::Args;
use clap::Parser;
use cups_rs::bindings::cupsSetUser;
use cups_rs::config::{EncryptionMode, set_encryption, set_server};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // `-h`: set server
    if let Some(server) = &args.server {
        set_server(Some(&server))?;
    }

    // -U: set user
    if let Some(username) = &args.username {
        let user = std::ffi::CString::new(username.as_str())?;
        unsafe { cupsSetUser(user.as_ptr()) };
    }

    // -E: if `-p`` present enable printer, otherwise require encryption
    if args.enable_encrypt {
        if let Some(printer) = &args.printer {
            ipp_helpers::enable_printer(printer)?;
        } else {
            set_encryption(EncryptionMode::Required);
        }
    }

    // `-c`: add printer to class
    if let Some(class) = &args.class {
        if !ipp_helpers::validate_name(class) {
            return Err("lpadmin: Class name can only contain printable characters.".into());
        }

        let printer = args
            .printer
            .as_deref()
            .ok_or("lpadmin: -c requires -p <destination>")?;

        ipp_helpers::add_printer_to_class(printer, class)?;
    }

    let needs_printer = args.device_uri.is_some()
        || args.model.is_some()
        || args.description.is_some()
        || args.location.is_some()
        || !args.options.is_empty()
        || !args.access_control.is_empty();

    if needs_printer && args.printer.is_none() {
        return Err("lpadmin: -p is required when using -v, -m, -D, -L, -o, or -u".into());
    }

    let opts: options::PrinterOptions = args.options.into();

    // `-p`: add/modify printer
    if let Some(printer) = &args.printer {
        ipp_helpers::add_or_modify_printer(
            printer,
            args.device_uri.as_deref(),
            args.model.as_deref(),
            args.description.as_deref(),
            args.location.as_deref(),
            &opts,
            &args.access_control,
        )?;
    }

    // `-d`: set default printer
    if let Some(default_printer) = &args.default_printer {
        ipp_helpers::set_default_printer(default_printer)?;
    }

    // `-x`: delete printer
    if let Some(printer) = &args.delete_printer {
        ipp_helpers::delete_printer(printer)?;
    }

    // `-r`: remove printer from class
    if let Some(class) = &args.remove_from_class {
        let printer = args
            .printer
            .as_deref()
            .ok_or("lpadmin: -r requires -p <destination>")?;

        ipp_helpers::delete_printer_from_class(printer, class)?;
    }

    // `-R`: remove option default
    if let Some(option) = &args.remove_option {
        let printer = args
            .printer
            .as_deref()
            .ok_or("lpadmin: -R requires -p <destination>")?;

        ipp_helpers::delete_printer_option(printer, option)?;
    }

    Ok(())
}
