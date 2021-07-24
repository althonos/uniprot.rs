use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

#[derive(Debug, Default, Clone)]
pub struct BiophysicochemicalProperties {
    pub absorption: Option<Absorption>,
    pub kinetics: Option<Kinetics>,
    pub ph_dependence: Option<String>,   // TODO: EvidenceString
    pub redox_potential: Option<String>, // TODO: EvidenceString
    pub temperature_dependence: Option<String>,
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct Absorption {
    pub max: Option<String>,  // FIXME: evidence string
    pub min: Option<String>,  // FIXME: evidence string
    pub text: Option<String>, // FIXME: evidence string
}

impl FromXml for Absorption {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"absorption");

        let mut absorption = Absorption::default();
        parse_inner! {event, reader, buffer,
            b"max" => {
                let max = reader.read_text(b"max", buffer)?;
                if absorption.max.replace(max).is_some() {
                    return Err(Error::DuplicateElement("max", "absorption"));
                }
            },
            b"min" => {
                let min = reader.read_text(b"min", buffer)?;
                if absorption.min.replace(min).is_some() {
                    return Err(Error::DuplicateElement("min", "absorption"));
                }
            },
            b"text" => {
                let text = reader.read_text(b"text", buffer)?;
                if absorption.text.replace(text).is_some() {
                    return Err(Error::DuplicateElement("text", "absorption"));
                }
            }
        }

        Ok(absorption)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct Kinetics {
    pub km: Vec<String>,      // FIXME: evidence string
    pub vmax: Vec<String>,    // FIXME: evidence string
    pub text: Option<String>, // FIXME: evidence string
}

impl FromXml for Kinetics {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"kinetics");

        let mut kinetics = Kinetics::default();
        parse_inner! {event, reader, buffer,
            b"KM" => {
                kinetics.km.push(reader.read_text(b"KM", buffer)?);
            },
            b"Vmax" => {
                kinetics.vmax.push(reader.read_text(b"Vmax", buffer)?);
            },
            b"text" => {
                let text = reader.read_text(b"text", buffer)?;
                if kinetics.text.replace(text).is_some() {
                    return Err(Error::DuplicateElement("text", "kinetics"));
                }
            }
        }

        Ok(kinetics)
    }
}
