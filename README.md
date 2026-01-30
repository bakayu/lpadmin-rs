# lpadmin-rs
Experimental Rust port of CUPS `lpadmin` using [`cups_rs`](https://github.com/Gmin2/cups-rs) bindings.

## Build
```
cargo build
```

## Usage
```
lpadmin-rs --help
lpadmin-rs -p printer -v device-uri -m everywhere
lpadmin-rs -p printer -D "Test Printer" -L "XYZ"
lpadmin-rs -c class -p printer
lpadmin-rs -r class -p printer
lpadmin-rs -x printer
lpadmin-rs -d printer
```

## Features
Implemented (non‑deprecated):
- `-p` add/modify printer
- `-d` set default printer
- `-x` delete printer
- `-c` add printer to class
- `-r` remove printer from class
- `-R` remove option default
- `-v` device URI
- `-D` description
- `-L` location
- `-m` model (e.g., `everywhere`)
- `-o` printer options (subset)
- `-u` access control
- `-E`, `-U`, `-h`

Not implemented:
- Deprecated flags (`-i`, `-P`)

## Notes
- Some behaviors depend on CUPS server configuration.
- Output formatting may differ from `lpadmin`.
