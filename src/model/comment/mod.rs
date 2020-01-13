pub mod alternative_product;
pub mod bpc_properties;
pub mod catalytic_activity;
pub mod cofactor;
pub mod conflict;
pub mod disease;
pub mod interaction;
pub mod mass_spectrometry;
pub mod online_information;
pub mod subcellular_location;

use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

use super::molecule::Molecule;
use super::feature_location::FeatureLocation;

use self::alternative_product::AlternativeProduct;
use self::bpc_properties::BiophysicochemicalProperties;
use self::catalytic_activity::CatalyticActivity;
use self::disease::Disease;
use self::online_information::OnlineInformation;
use self::subcellular_location::SubcellularLocation;
use self::interaction::Interaction;
use self::mass_spectrometry::MassSpectrometry;
use self::cofactor::Cofactor;
use self::conflict::Conflict;

#[derive(Debug, Clone)]
/// Describes different types of general annotations.
pub struct Comment {
    // fields
    pub molecule: Option<Molecule>,
    // location: Vec<Location>,
    pub text: Vec<String>,              // FIXME: type should be evidence text?
    pub ty: CommentType,
    pub evidences: Vec<usize>,            // TODO: extract evidence attribute
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

        macro_rules! parse_comment {
            (
                $event:expr,
                $reader:expr,
                $buffer:expr,
                $comment:expr
                $(, $e:ident @ $l:expr => $r:expr )*
            ) => ({
                parse_inner!{$event, $reader, $buffer,
                    $( $e @ $l => $r ,)*
                    e @ b"text" => {
                        $comment.text.push($reader.read_text(b"text", $buffer)?);
                    },
                    e @ b"molecule" => {
                        $comment.molecule = Molecule::from_xml(&e, $reader, $buffer)
                            .map(Some)?;
                    }
                }
            })
        }

        let attr = attributes_to_hashmap(event)?;
        let mut comment = Comment::new(CommentType::Miscellaneous);
        comment.evidences = get_evidences(reader, &attr)?;

        match attr.get(&b"type"[..]).map(|x| &*x.value) {
            Some(b"function") => {
                comment.ty = CommentType::Function;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"similarity") => {
                comment.ty = CommentType::Similarity;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"subunit") => {
                comment.ty = CommentType::Subunit;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"PTM") => {
                comment.ty = CommentType::Ptm;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"developmental stage") => {
                comment.ty = CommentType::DevelopmentalStage;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"disruption phenotype") => {
                comment.ty = CommentType::DisruptionPhenotype;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"tissue specificity") => {
                comment.ty = CommentType::TissueSpecificity;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"miscellaneous") => {
                comment.ty = CommentType::Miscellaneous;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"induction") => {
                comment.ty = CommentType::Induction;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"caution") => {
                comment.ty = CommentType::Caution;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"pathway") => {
                comment.ty = CommentType::Pathway;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"toxic dose") => {
                comment.ty = CommentType::ToxicDose;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"activity regulation") => {
                comment.ty = CommentType::ActivityRegulation;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"domain") => {
                comment.ty = CommentType::Domain;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"biotechnology") => {
                comment.ty = CommentType::Biotechnology;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"polymorphism") => {
                comment.ty = CommentType::Polymorphism;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"pharmaceutical") => {
                comment.ty = CommentType::Pharmaceutical;
                parse_comment!{event, reader, buffer, comment}
            }
            Some(b"allergen") => {
                comment.ty = CommentType::Allergen;
                parse_comment!{event, reader, buffer, comment}
            }

            Some(b"subcellular location") => {
                let mut locations = Vec::new();
                parse_comment!{event, reader, buffer, comment,
                    e @ b"subcellularLocation" => {
                        locations.push(FromXml::from_xml(&e, reader, buffer)?);
                    }
                }
                comment.ty = CommentType::SubcellularLocation(locations);
            }

            // Some(b"alternative products") => {
            //     let mut product = AlternativeProduct::default();
            //     comment_loop!{self, b, buffer, comment,
            //         e @ b"event" => {
            //             product.events.push(self.extract_alternative_products_event(e)?);
            //         },
            //         e @ b"isoform" => {
            //             product.isoforms.push(self.extract_alternative_products_isoform(e)?);
            //         }
            //     }
            //     comment.ty = CommentType::AlternativeProduct(product);
            // }
            //
            // Some(b"interaction") => {
            //     let mut organisms_differ = false;
            //     let mut experiments = None;
            //     let mut interactants = Vec::new();
            //
            //     // extract interaction elements
            //     comment_loop!{self, b, buffer, comment,
            //         e @ b"interactant" => {
            //             interactants.push(self.extract_interactant(e)?);
            //         },
            //         e @ b"organismsDiffer" => {
            //             let text = self.xml.read_text(b"organismsDiffer", &mut buffer)?;
            //             organisms_differ = bool::from_str(&text)
            //                 .expect("ERR: could not parse `organismsDiffer` as bool");
            //         },
            //         e @ b"experiments" => {
            //             let text = self.xml.read_text(b"experiments", &mut buffer)?;
            //             experiments = usize::from_str(&text)
            //                 .map(Some)
            //                 .expect("ERR: could not parse `experiments` as usize");
            //         }
            //     }
            //
            //     // check that we have 2 interactants
            //     let i2 = interactants.pop().expect("ERR: missing `interactant` in `interaction`");
            //     let i1 = interactants.pop().expect("ERR: missing `interactant` in `interaction`");
            //     if !interactants.is_empty() {
            //         panic!("ERR: too many `interactant` in `interaction`");
            //     }
            //
            //     // create new interaction
            //     comment.ty = CommentType::Interaction(Interaction {
            //         organisms_differ,
            //         experiments: experiments
            //             .expect("ERR: missing `experiments` in `interaction`"),
            //         interactants: (i1, i2),
            //     });
            // }
            //
            // Some(b"sequence caution") => {
            //     let mut optconflict = None;
            //
            //     // extract inner `conflict`
            //     comment_loop!{self, b, buffer, comment,
            //         e @ b"conflict" => {
            //             let conflict = self.extract_conflict(e)?;
            //             if let Some(_) = optconflict.replace(conflict) {
            //                 panic!("ERR: duplicate `conflict` in `sequence caution`")
            //             }
            //         }
            //     }
            //
            //     // check a `conflict` was extracted
            //     comment.ty = optconflict.map(CommentType::SequenceCaution)
            //         .expect("ERR: missing `conflict` in `sequence caution`");
            // }
            //
            // Some(b"mass spectrometry") => {
            //     let mut ms = MassSpectrometry::default();
            //     ms.mass = attr.get(&b"mass"[..])
            //         .map(|x| x.unescape_and_decode_value(&mut self.xml))
            //         .transpose()?
            //         .map(|s| f64::from_str(&s))
            //         .transpose()
            //         .expect("could not parse `mass` as f64");
            //     ms.error = attr.get(&b"error"[..])
            //         .map(|x| x.unescape_and_decode_value(&mut self.xml))
            //         .transpose()?;
            //     ms.method = attr.get(&b"method"[..])
            //         .map(|x| x.unescape_and_decode_value(&mut self.xml))
            //         .transpose()?;
            //
            //     self.xml.read_to_end(b"comment", &mut buffer)?;
            //     comment.ty = CommentType::MassSpectrometry(ms);
            // }
            //
            // Some(b"disease") => {
            //     let mut optdisease = None;
            //     comment_loop!{self, b, buffer, comment,
            //         e @ b"disease" => {
            //             let disease = self.extract_disease(e)?;
            //             if let Some(_) = optdisease.replace(disease) {
            //                 panic!("ERR: duplicate `disease` in `comment`")
            //             }
            //         }
            //     }
            //     comment.ty = CommentType::Disease(optdisease);
            // }
            //
            // Some(b"biophysicochemical properties") => {
            //     let mut bcp = BiophysicochemicalProperties::default();
            //     comment_loop!{self, b, buffer, comment,
            //         e @ b"absorption" => {
            //             let absorption = self.extract_absorption(e)?;
            //             if let Some(_) = bcp.absorption.replace(absorption) {
            //                 panic!("ERR: duplicate `absorption` in `comment`")
            //             }
            //         },
            //         e @ b"kinetics" => {
            //             let kinetics = self.extract_kinetics(e)?;
            //             if let Some(_) = bcp.kinetics.replace(kinetics) {
            //                 panic!("ERR: duplicate `kinetics` in `comment`")
            //             }
            //         },
            //         e @ b"phDependence" => {
            //             state_loop!{self, e, self.buffer,
            //                 t @ b"text" => {
            //                     let text = self.xml.read_text(b"text", &mut self.buffer)?;
            //                     bcp.ph_dependence = Some(text);
            //                 }
            //             }
            //         },
            //         e @ b"redoxPotential" => {
            //             state_loop!{self, e, self.buffer,
            //                 t @ b"text" => {
            //                     let text = self.xml.read_text(b"text", &mut self.buffer)?;
            //                     bcp.redox_potential = Some(text);
            //                 }
            //             }
            //         },
            //         e @ b"temperatureDependence" => {
            //             state_loop!{self, e, self.buffer,
            //                 t @ b"text" => {
            //                     let text = self.xml.read_text(b"text", &mut self.buffer)?;
            //                     bcp.temperature_dependence = Some(text);
            //                 }
            //             }
            //         }
            //     }
            //     comment.ty = CommentType::BiophysicochemicalProperties(bcp);
            // }
            //
            // Some(b"catalytic activity") => {
            //     let mut physio = Vec::new();
            //     let mut optreact = None;
            //
            //     comment_loop!{self, b, buffer, comment,
            //         e @ b"reaction" => {
            //             let reaction = self.extract_reaction(e)?;
            //             if let Some(_) = optreact.replace(reaction) {
            //                 panic!("ERR: duplicate `reaction` in `comment`")
            //             }
            //         },
            //         e @ b"physiologicalReaction" => {
            //             physio.push(self.extract_physiological_reaction(e)?);
            //
            //         }
            //     }
            //
            //     let mut act = optreact.map(CatalyticActivity::new)
            //         .expect("ERR: could not find required `reaction` in `comment`");
            //
            //     if physio.len() > 2 {
            //         panic!("ERR: too many `physiologicalReaction` found in `comment`")
            //     }
            //     act.physiological_reactions = physio;
            //     comment.ty = CommentType::CatalyticActivity(act);
            // }
            //
            // Some(b"online information") => {
            //     let mut info = OnlineInformation::default();
            //     info.name = attr.get(&b"name"[..])
            //         .map(|a| a.unescape_and_decode_value(&self.xml))
            //         .transpose()?;
            //
            //     comment_loop!{self, b, buffer, comment,
            //         e @ b"link" => {
            //             let uri = e.attributes()
            //                 .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"uri").unwrap_or_default())
            //                 .transpose()?
            //                 .map(|a| a.unescape_and_decode_value(&mut self.xml))
            //                 .transpose()?
            //                 .map(|s| url::Url::parse(&s))
            //                 .expect("ERR: could not find required `uri` on `link`")
            //                 .expect("ERR: could not parse `uri` as url::Url");
            //             info.links.push(uri);
            //             self.xml.read_to_end(b"link", &mut buffer)?;
            //         }
            //     }
            //
            //     comment.ty = CommentType::OnlineInformation(info);
            // }
            //
            // Some(b"cofactor") => {
            //     let mut cofactors = Vec::new();
            //     comment_loop!{self, b, buffer, comment,
            //         e @ b"cofactor" => cofactors.push(self.extract_cofactor(e)?)
            //     }
            //     comment.ty = CommentType::Cofactor(cofactors)
            // }
            //
            // Some(b"RNA editing") => {
            //     let mut locations = Vec::new();
            //     comment_loop!{self, b, buffer, comment,
            //         e @ b"location" => locations.push(self.extract_feature_location(e)?)
            //     }
            //     comment.ty = CommentType::RnaEditing(locations);
            // }

            Some(other) => panic!("unknown `type` in `comment`: {:?}", String::from_utf8_lossy(other)),
            None => panic!("could not find required `type` attribute on `comment`"),
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
