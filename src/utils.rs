//! Utility functions.

use cretonne_codegen::isa;
use cretonne_codegen::isa::TargetIsa;
use cretonne_codegen::settings::{self, FlagsOrIsa};
use cretonne_reader::{parse_options, Location};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::str::FromStr;
use target_lexicon::Triple;

/// Read an entire file into a string.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Read an entire file into a vector of bytes.
pub fn read_to_end<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Like `FlagsOrIsa`, but holds ownership.
pub enum OwnedFlagsOrIsa {
    Flags(settings::Flags),
    Isa(Box<TargetIsa>),
}

impl OwnedFlagsOrIsa {
    /// Produce a FlagsOrIsa reference.
    pub fn as_fisa(&self) -> FlagsOrIsa {
        match *self {
            OwnedFlagsOrIsa::Flags(ref flags) => FlagsOrIsa::from(flags),
            OwnedFlagsOrIsa::Isa(ref isa) => FlagsOrIsa::from(&**isa),
        }
    }
}

/// Parse "set" and "triple" commands.
pub fn parse_sets_and_triple(
    flag_set: &[String],
    flag_triple: &str,
) -> Result<OwnedFlagsOrIsa, String> {
    let mut flag_builder = settings::builder();
    parse_options(
        flag_set.iter().map(|x| x.as_str()),
        &mut flag_builder,
        Location { line_number: 0 },
    ).map_err(|err| err.to_string())?;

    let mut words = flag_triple.trim().split_whitespace();
    // Look for `target foo`.
    if let Some(triple_name) = words.next() {
        let triple = match Triple::from_str(triple_name) {
            Ok(triple) => triple,
            Err(parse_error) => return Err(parse_error.to_string()),
        };
        let mut isa_builder = isa::lookup(triple).map_err(|err| match err {
            isa::LookupError::SupportDisabled => {
                format!("support for triple '{}' is disabled", triple_name)
            }
            isa::LookupError::Unsupported => format!(
                "support for triple '{}' is not implemented yet",
                triple_name
            ),
        })?;
        // Apply the ISA-specific settings to `isa_builder`.
        parse_options(words, &mut isa_builder, Location { line_number: 0 })
            .map_err(|err| err.to_string())?;

        Ok(OwnedFlagsOrIsa::Isa(
            isa_builder.finish(settings::Flags::new(flag_builder)),
        ))
    } else {
        Ok(OwnedFlagsOrIsa::Flags(settings::Flags::new(flag_builder)))
    }
}
