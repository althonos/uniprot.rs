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
        debug_assert_eq!(event.local_name().as_ref(), b"absorption");

        let mut absorption = Absorption::default();
        parse_inner! {event, reader, buffer,
            e @ b"max" => {
                let max = parse_text!(e, reader, buffer);
                if absorption.max.replace(max).is_some() {
                    return Err(Error::DuplicateElement("max", "absorption"));
                }
            },
            e @ b"min" => {
                let min = parse_text!(e, reader, buffer);
                if absorption.min.replace(min).is_some() {
                    return Err(Error::DuplicateElement("min", "absorption"));
                }
            },
            e @ b"text" => {
                let text = parse_text!(e, reader, buffer);
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
        debug_assert_eq!(event.local_name().as_ref(), b"kinetics");

        let mut kinetics = Kinetics::default();
        parse_inner! {event, reader, buffer,
            e @ b"KM" => {
                kinetics.km.push(parse_text!(e, reader, buffer));
            },
            e @ b"Vmax" => {
                kinetics.vmax.push(parse_text!(e, reader, buffer));
            },
            e @ b"text" => {
                let text = parse_text!(e, reader, buffer);
                if kinetics.text.replace(text).is_some() {
                    return Err(Error::DuplicateElement("text", "kinetics"));
                }
            }
        }

        Ok(kinetics)
    }
}
