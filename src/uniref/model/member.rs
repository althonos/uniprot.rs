use std::io::BufRead;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::common::ShortString;
use crate::error::Error;
use crate::parser::FromXml;

use super::Reference;
use super::Sequence;

/// A UniRef cluster member.
#[derive(Debug, Clone)]
pub struct Member {
    pub sequence: Option<Sequence>,
    pub db_reference: Reference,
}

impl FromXml for Member {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert!(
            event.local_name().as_ref() == b"member"
                || event.local_name().as_ref() == b"representativeMember"
        );

        let mut sequence = None;
        let mut dbref = None;

        parse_inner! {event, reader, buffer,
            e @ b"sequence" => {
                sequence = Some(FromXml::from_xml(&e, reader, buffer)?);
            },
            e @ b"dbReference" => {
                dbref = Some(FromXml::from_xml(&e, reader, buffer)?);
            }
        }

        Ok(Member {
            sequence,
            db_reference: dbref.ok_or(Error::MissingElement("dbReference", "member"))?,
        })
    }
}
