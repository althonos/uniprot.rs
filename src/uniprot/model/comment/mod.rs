//! The different kind of annotations that can be attached to an entry.

mod alternative_product;
mod bpc_properties;
mod catalytic_activity;
mod cofactor;
mod conflict;
mod disease;
mod interaction;
mod mass_spectrometry;
mod online_information;
mod subcellular_location;

use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

use super::feature_location::FeatureLocation;
use super::molecule::Molecule;

pub use self::alternative_product::AlternativeProduct;
pub use self::alternative_product::Event;
pub use self::alternative_product::Isoform;
pub use self::alternative_product::IsoformSequence;
pub use self::alternative_product::IsoformSequenceType;
pub use self::bpc_properties::BiophysicochemicalProperties;
pub use self::bpc_properties::Absorption;
pub use self::bpc_properties::Kinetics;
pub use self::catalytic_activity::CatalyticActivity;
pub use self::catalytic_activity::Reaction;
pub use self::catalytic_activity::PhysiologicalReaction;
pub use self::catalytic_activity::Direction;
pub use self::cofactor::Cofactor;
pub use self::conflict::Conflict;
pub use self::conflict::ConflictType;
pub use self::conflict::ConflictSequence;
pub use self::conflict::Resource;
pub use self::disease::Disease;
pub use self::interaction::Interactant;
pub use self::interaction::Interaction;
pub use self::mass_spectrometry::MassSpectrometry;
pub use self::online_information::OnlineInformation;
pub use self::subcellular_location::SubcellularLocation;

#[derive(Debug, Clone)]
/// Describes different types of general annotations.
pub struct Comment {
    // fields
    pub molecule: Option<Molecule>,
    // location: Vec<Location>,
    pub text: Vec<String>, // FIXME: type should be evidence text?
    pub ty: CommentType,
    pub evidences: Vec<usize>, // TODO: extract evidence attribute
}

impl Comment {
    pub fn new(ty: CommentType) -> Self {
        Self {
            ty,
            molecule: Default::default(),
            text: Default::default(),
            evidences: Default::default(),
        }
    }
}

impl FromXml for Comment {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"comment");

        let attr = attributes_to_hashmap(event)?;
        let mut comment = Comment::new(CommentType::Miscellaneous);
        comment.evidences = get_evidences(reader, &attr)?;

        match attr.get(&b"type"[..]).map(|x| &*x.value) {
            Some(b"function") => {
                comment.ty = CommentType::Function;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"similarity") => {
                comment.ty = CommentType::Similarity;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"subunit") => {
                comment.ty = CommentType::Subunit;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"PTM") => {
                comment.ty = CommentType::Ptm;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"developmental stage") => {
                comment.ty = CommentType::DevelopmentalStage;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"disruption phenotype") => {
                comment.ty = CommentType::DisruptionPhenotype;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"tissue specificity") => {
                comment.ty = CommentType::TissueSpecificity;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"miscellaneous") => {
                comment.ty = CommentType::Miscellaneous;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"induction") => {
                comment.ty = CommentType::Induction;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"caution") => {
                comment.ty = CommentType::Caution;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"pathway") => {
                comment.ty = CommentType::Pathway;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"toxic dose") => {
                comment.ty = CommentType::ToxicDose;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"activity regulation") => {
                comment.ty = CommentType::ActivityRegulation;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"domain") => {
                comment.ty = CommentType::Domain;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"biotechnology") => {
                comment.ty = CommentType::Biotechnology;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"polymorphism") => {
                comment.ty = CommentType::Polymorphism;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"pharmaceutical") => {
                comment.ty = CommentType::Pharmaceutical;
                parse_comment! {event, reader, buffer, comment}
            }
            Some(b"allergen") => {
                comment.ty = CommentType::Allergen;
                parse_comment! {event, reader, buffer, comment}
            }

            Some(b"subcellular location") => {
                let mut locations = Vec::new();
                parse_comment! {event, reader, buffer, comment,
                    e @ b"subcellularLocation" => {
                        locations.push(FromXml::from_xml(&e, reader, buffer)?);
                    }
                }
                comment.ty = CommentType::SubcellularLocation(locations);
            }

            Some(b"alternative products") => {
                let mut product = AlternativeProduct::default();
                parse_comment! {event, reader, buffer, comment,
                    e @ b"event" => {
                        product.events.push(FromXml::from_xml(&e, reader, buffer)?);
                    },
                    e @ b"isoform" => {
                        product.isoforms.push(FromXml::from_xml(&e, reader, buffer)?);
                    }
                }
                comment.ty = CommentType::AlternativeProduct(product);
            }

            Some(b"interaction") => {
                let mut organisms_differ = false;
                let mut experiments = None;
                let mut interactants = Vec::new();

                // extract interaction elements
                parse_comment! {event, reader, buffer, comment,
                    e @ b"interactant" => {
                        interactants.push(Interactant::from_xml(&e, reader, buffer)?);
                    },
                    b"organismsDiffer" => {
                        let text = reader.read_text(b"organismsDiffer", buffer)?;
                        organisms_differ = bool::from_str(&text)?;
                    },
                    b"experiments" => {
                        let text = reader.read_text(b"experiments", buffer)?;
                        experiments = usize::from_str(&text).map(Some)?;
                    }
                }

                // check that we have 2 interactants
                let i2 = interactants
                    .pop()
                    .ok_or(Error::MissingElement("interactant", "interaction"))?;
                let i1 = interactants
                    .pop()
                    .ok_or(Error::MissingElement("interactant", "interaction"))?;
                if !interactants.is_empty() {
                    return Err(Error::DuplicateElement("interactant", "interaction"));
                }

                // create new interaction
                comment.ty = CommentType::Interaction(Interaction {
                    organisms_differ,
                    interactants: (i1, i2),
                    experiments: experiments
                        .ok_or(Error::MissingElement("experiments", "interaction"))?,
                });
            }

            Some(b"sequence caution") => {
                let mut optconflict = None;

                // extract inner `conflict`
                parse_comment! {event, reader, buffer, comment,
                    e @ b"conflict" => {
                        let conflict = FromXml::from_xml(&e, reader, buffer)?;
                        if optconflict.replace(conflict).is_some() {
                            return Err(Error::DuplicateElement("conflict", "comment"));
                        }
                    }
                }

                // check a `conflict` was extracted
                comment.ty = optconflict
                    .map(CommentType::SequenceCaution)
                    .ok_or(Error::MissingElement("conflict", "sequence caution"))?
            }

            Some(b"mass spectrometry") => {
                let mut ms = MassSpectrometry::default();
                ms.mass = attr
                    .get(&b"mass"[..])
                    .map(|x| x.unescape_and_decode_value(reader))
                    .transpose()?
                    .map(|s| f64::from_str(&s))
                    .transpose()
                    .expect("could not parse `mass` as f64");
                ms.error = attr
                    .get(&b"error"[..])
                    .map(|x| x.unescape_and_decode_value(reader))
                    .transpose()?;
                ms.method = attr
                    .get(&b"method"[..])
                    .map(|x| x.unescape_and_decode_value(reader))
                    .transpose()?;

                parse_comment! {event, reader, buffer, comment}
                comment.ty = CommentType::MassSpectrometry(ms);
            }

            Some(b"disease") => {
                let mut optdisease = None;
                parse_comment! {event, reader, buffer, comment,
                    e @ b"disease" => {
                        let disease = FromXml::from_xml(&e, reader, buffer)?;
                        if optdisease.replace(disease).is_some() {
                            return Err(Error::DuplicateElement("disease", "comment"));
                        }
                    }
                }
                comment.ty = CommentType::Disease(optdisease);
            }

            Some(b"biophysicochemical properties") => {
                let mut bcp = BiophysicochemicalProperties::default();
                parse_comment! {event, reader, buffer, comment,
                    e @ b"absorption" => {
                        let absorption = FromXml::from_xml(&e, reader, buffer)?;
                        if bcp.absorption.replace(absorption).is_some() {
                            return Err(Error::DuplicateElement("absorption", "comment"));
                        }
                    },
                    e @ b"kinetics" => {
                        let kinetics = FromXml::from_xml(&e, reader, buffer)?;
                        if bcp.kinetics.replace(kinetics).is_some() {
                            return Err(Error::DuplicateElement("kinetics", "comment"));
                        }
                    },
                    e @ b"phDependence" => {
                        parse_inner!{e, reader, buffer,
                            b"text" => {
                                let text = reader.read_text(b"text", buffer)?;
                                bcp.ph_dependence = Some(text);
                            }
                        }
                    },
                    e @ b"redoxPotential" => {
                        parse_inner!{e, reader, buffer,
                            b"text" => {
                                let text = reader.read_text(b"text", buffer)?;
                                bcp.redox_potential = Some(text);
                            }
                        }
                    },
                    e @ b"temperatureDependence" => {
                        parse_inner!{e, reader, buffer,
                            b"text" => {
                                let text = reader.read_text(b"text", buffer)?;
                                bcp.temperature_dependence = Some(text);
                            }
                        }
                    }
                }
                comment.ty = CommentType::BiophysicochemicalProperties(bcp);
            }

            Some(b"catalytic activity") => {
                let mut physio = Vec::new();
                let mut optreact = None;

                parse_comment! {event, reader, buffer, comment,
                    e @ b"reaction" => {
                        let reaction = FromXml::from_xml(&e, reader, buffer)?;
                        if optreact.replace(reaction).is_some() {
                            return Err(Error::DuplicateElement("reaction", "comment"));
                        }
                    },
                    e @ b"physiologicalReaction" => {
                        physio.push(FromXml::from_xml(&e, reader, buffer)?);
                    }
                }

                let mut act = optreact
                    .map(CatalyticActivity::new)
                    .ok_or(Error::MissingElement("reaction", "comment"))?;

                if physio.len() > 2 {
                    return Err(Error::DuplicateElement("physiologicalReaction", "comment"));
                }
                act.physiological_reactions = physio;
                comment.ty = CommentType::CatalyticActivity(act);
            }

            Some(b"online information") => {
                let mut info = OnlineInformation::default();
                info.name = attr
                    .get(&b"name"[..])
                    .map(|a| a.unescape_and_decode_value(reader))
                    .transpose()?;

                parse_comment! {event, reader, buffer, comment,
                    e @ b"link" => {
                        let uri = e.attributes()
                            .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"uri").unwrap_or_default())
                            .transpose()?
                            .map(|a| a.unescape_and_decode_value(reader))
                            .transpose()?
                            .map(|s| url::Url::parse(&s))
                            .ok_or(Error::MissingElement("uri", "link"))??;
                        info.links.push(uri);
                        reader.read_to_end(b"link", buffer)?;
                    }
                }

                comment.ty = CommentType::OnlineInformation(info);
            }

            Some(b"cofactor") => {
                let mut cofactors = Vec::new();
                parse_comment! {event, reader, buffer, comment,
                    e @ b"cofactor" => {
                        cofactors.push(FromXml::from_xml(&e, reader, buffer)?);
                    }
                }
                comment.ty = CommentType::Cofactor(cofactors)
            }

            Some(b"RNA editing") => {
                let mut locations = Vec::new();
                parse_comment! {event, reader, buffer, comment,
                    e @ b"location" => {
                        locations.push(FromXml::from_xml(&e, reader, buffer)?);
                    }
                }
                comment.ty = CommentType::RnaEditing(locations);
            }

            None => return Err(Error::MissingAttribute("type", "comment")),
            Some(other) => {
                return Err(Error::invalid_value(
                    "type",
                    "comment",
                    String::from_utf8_lossy(other),
                ))
            }
        }

        Ok(comment)
    }
}

#[derive(Debug, Clone)]
pub enum CommentType {
    Allergen,
    AlternativeProduct(AlternativeProduct),
    Biotechnology,
    BiophysicochemicalProperties(BiophysicochemicalProperties),
    CatalyticActivity(CatalyticActivity),
    Caution,
    Cofactor(Vec<Cofactor>),
    DevelopmentalStage,
    Disease(Option<Disease>),
    Domain,
    DisruptionPhenotype,
    ActivityRegulation,
    Function,
    Induction,
    Miscellaneous,
    Pathway,
    Pharmaceutical,
    Polymorphism,
    Ptm,
    RnaEditing(Vec<FeatureLocation>), // FIXME: possible dedicated type
    Similarity,
    SubcellularLocation(Vec<SubcellularLocation>),
    SequenceCaution(Conflict),
    Subunit,
    TissueSpecificity,
    ToxicDose,
    OnlineInformation(OnlineInformation),
    MassSpectrometry(MassSpectrometry),
    Interaction(Interaction),
}
