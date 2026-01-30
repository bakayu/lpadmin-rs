use crate::access_control::{AccessControl, parse_access_control};
use crate::options::{PrinterOption, parse_printer_option};
use clap::Parser;

/// Configure CUPS printers and classes
#[derive(Parser, Debug)]
#[command(name = "lpadmin-rs")]
#[command(version)]
#[command(disable_help_flag = true)]
pub struct Args {
    /// Print help information
    #[arg(long = "help", action = clap::ArgAction::Help)]
    pub help: (),

    /// Set the named destination as the server default
    #[arg(short = 'd', value_name = "DESTINATION")]
    pub default_printer: Option<String>,

    /// Specify/add the named destination
    #[arg(short = 'p', value_name = "DESTINATION")]
    pub printer: Option<String>,

    /// Remove the named destination
    #[arg(short = 'x', value_name = "DESTINATION")]
    pub delete_printer: Option<String>,

    /// Add the named destination to a class
    #[arg(short = 'c', value_name = "CLASS")]
    pub class: Option<String>,

    /// Specify the textual description of the printer
    #[arg(short = 'D', value_name = "DESCRIPTION")]
    pub description: Option<String>,

    /// Encrypt the connection to the server (before -p)
    #[arg(
        short = 'E',
        help = "Encrypt the connection to the server (if used without -p)\nEnable and accept jobs on the printer (if used with -p)"
    )]
    pub enable_encrypt: bool,

    /// Connect to the named server and port
    #[arg(short = 'h', value_name = "SERVER[:PORT]")]
    pub server: Option<String>,

    /// Specify the textual location of the printer
    #[arg(short = 'L', value_name = "LOCATION")]
    pub location: Option<String>,

    /// Specify a standard model/PPD file for the printer
    #[arg(
        short = 'm',
        value_name = "MODEL",
        help = "Specify a standard model/PPD file for the printer\nUse \"everywhere\" for IPP Everywhere compatible printers"
    )]
    pub model: Option<String>,

    /// Set printer options (repeatable)
    #[arg(short = 'o', value_name = "NAME=VALUE", value_parser = parse_printer_option)]
    pub options: Vec<PrinterOption>,

    /// Remove the named destination from a class
    #[arg(short = 'r', value_name = "CLASS")]
    pub remove_from_class: Option<String>,

    /// Remove the default value for the named option
    #[arg(short = 'R', value_name = "NAME-DEFAULT")]
    pub remove_option: Option<String>,

    /// Set user-level access control
    #[arg(short = 'u', value_name = "allow:|deny:...")]
    #[arg(value_parser = parse_access_control)]
    pub access_control: Vec<AccessControl>,

    /// Specify the username to use for authentication
    #[arg(short = 'U', value_name = "USERNAME")]
    pub username: Option<String>,

    /// Specify the device URI for the printer
    #[arg(short = 'v', value_name = "DEVICE-URI")]
    pub device_uri: Option<String>,
}
