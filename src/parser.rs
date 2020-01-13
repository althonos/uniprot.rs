use std::collections::HashSet;
use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;

use super::model::*;
use super::error::Error;

macro_rules! parse_inner {
    ($start:expr, $reader:expr, $buffer:expr, $( $e:ident @ $l:expr => $r:expr ),*  ) => ({
        loop {
            use $crate::quick_xml::events::BytesEnd;
            use $crate::quick_xml::events::BytesStart;
            use $crate::quick_xml::events::Event;
            use $crate::quick_xml::Error as XmlError;

            $buffer.clear();
            match $reader.read_event($buffer) {
                $( Ok(Event::Start(ref e)) if e.local_name() == $l => {
                    let $e = e.clone().into_owned();
                    $r
                }),*
                Ok(Event::Start(ref e)) => {
                    $reader.read_to_end(e.local_name(), &mut Vec::new())?;
                    // unimplemented!(
                    //     "`{}` in `{}`",
                    //     String::from_utf8_lossy(e.local_name()),
                    //     String::from_utf8_lossy($start.local_name())
                    // );
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(Event::Eof) => {
                    let e = String::from_utf8_lossy($start.local_name());
                    return Err(XmlError::UnexpectedEof(e.to_string()));
                }
                Ok(Event::End(ref e)) if e.local_name() == $start.local_name() => {
                    break;
                }
                _ => continue,
            }
        }
    })
}

// ---------------------------------------------------------------------------

pub(crate) mod utils {
    use std::collections::HashMap;
    use std::io::BufRead;
    use std::str::FromStr;

    use quick_xml::Reader;
    use quick_xml::Error as XmlError;
    use quick_xml::events::attributes::Attribute;
    use quick_xml::events::BytesStart;

    pub(crate) fn attributes_to_hashmap<'a>(b: &'a BytesStart<'a>) -> Result<HashMap<&'a [u8], Attribute<'a>>, XmlError> {
        b.attributes().map(|r| r.map(|a| (a.key, a))).collect()
    }

    pub(crate) fn get_evidences<'a, B: BufRead>(reader: &mut Reader<B>, attr: &HashMap<&'a [u8], Attribute<'a>>) -> Result<Vec<usize>, XmlError> {
        Ok(attr.get(&b"evidence"[..])
            .map(|a| a.unescape_and_decode_value(reader))
            .transpose()?
            .map(|e| e.split(' ').map(usize::from_str).collect::<Result<Vec<_>, _>>())
            .transpose()
            .expect("ERR: could not decode evidence number")
            .unwrap_or_default())
    }
}

// ---------------------------------------------------------------------------

pub struct UniprotParser<B: BufRead> {
    xml: Reader<B>,
    buffer: Vec<u8>,
    cache: Option<<Self as Iterator>::Item>,
    finished: bool,
    ignores: HashSet<&'static [u8]>,
}

impl<B: BufRead> UniprotParser<B> {
    pub fn new(reader: B) -> UniprotParser<B> {
        let mut buffer = Vec::new();
        let mut finished = false;
        let mut ignores = HashSet::new();
        let mut xml = Reader::from_reader(reader);
        xml.expand_empty_elements(true);

        // read until we enter the `uniprot` element
        let cache = loop {
            buffer.clear();
            match xml.read_event(&mut buffer) {
                Ok(Event::Start(ref e)) if e.local_name() == b"uniprot" => break None,
                Err(e) => break Some(Err(e)),
                Ok(Event::Eof) => {
                    let e = String::from("xml");
                    break Some(Err(XmlError::UnexpectedEof(e)));
                }
                _ => (),
            }
        };

        UniprotParser { xml, buffer, cache, finished, ignores }
    }
}

impl<B: BufRead> Iterator for UniprotParser<B> {
    type Item = Result<Entry, XmlError>;
    fn next(&mut self) -> Option<Self::Item> {
        // return cached item if any
        if let Some(item) = self.cache.take() {
            return Some(item);
        }

        // if finished, simply return `None`
        if self.finished {
            return None;
        }

        // enter the next `entry` element
        loop {
            self.buffer.clear();
            match self.xml.read_event(&mut self.buffer) {
                // if an error is raised, return it
                Err(e) => return Some(Err(e)),
                // error if reaching EOF
                Ok(Event::Eof) => {
                    let e = String::from("entry");
                    return Some(Err(XmlError::UnexpectedEof(e)));
                }
                // if end of `uniprot` is reached, return no further item
                Ok(Event::End(ref e)) if e.local_name() == b"uniprot" => {
                    self.finished = true;
                    return None;
                },
                // create a new Entry
                Ok(Event::Start(ref e)) if e.local_name() == b"entry" => {
                    let event = e.clone().into_owned();
                    return Entry::from_xml(&event, &mut self.xml, &mut self.buffer)
                        .map(Some).transpose();
                },
                _ => (),
            }
        };
    }
}

// -----------------------------------------------------------------------

pub(crate) trait FromXml: Sized {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error>;
}





/*
impl<B: BufRead> UniprotParser<B> {

    fn make_attrs<'a>(&self, b: &'a BytesStart<'a>) -> Result<HashMap<&'a [u8], Attribute<'a>>, XmlError> {
        b.attributes().map(|r| r.map(|a| (a.key, a))).collect()
    }

    fn get_evidences<'a>(&mut self, attr: &HashMap<&'a [u8], Attribute<'a>>) -> Result<Vec<usize>, XmlError> {
        Ok(attr.get(&b"evidence"[..])
            .map(|a| a.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|e| e.split(' ').map(usize::from_str).collect::<Result<Vec<_>, _>>())
            .transpose()
            .expect("ERR: could not decode evidence number")
            .unwrap_or_default())
    }

    // -----------------------------------------------------------------------

    fn extract_absorption(&mut self, b: &BytesStart) -> Result<Absorption, XmlError> {
        debug_assert_eq!(b.local_name(), b"absorption");

        let mut absorption = Absorption::default();

        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"max" => {
                let max = self.xml.read_text(b"max", &mut buffer)?;
                if let Some(_) = absorption.max.replace(max) {
                    panic!("ERR: duplicate `max` in `absorption`");
                }
            },
            e @ b"min" => {
                let min = self.xml.read_text(b"min", &mut buffer)?;
                if let Some(_) = absorption.min.replace(min) {
                    panic!("ERR: duplicate `min` in `absorption`");
                }
            },
            e @ b"text" => {
                let text = self.xml.read_text(b"text", &mut buffer)?;
                if let Some(_) = absorption.text.replace(text) {
                    panic!("ERR: duplicate `text` in `absorption`");
                }
            }
        }

        Ok(absorption)
    }

    fn extract_kinetics(&mut self, b: &BytesStart) -> Result<Kinetics, XmlError> {
        debug_assert_eq!(b.local_name(), b"kinetics");

        let mut kinetics = Kinetics::default();

        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"KM" => {
                kinetics.km.push(self.xml.read_text(b"KM", &mut buffer)?);
            },
            e @ b"Vmax" => {
                kinetics.vmax.push(self.xml.read_text(b"Vmax", &mut buffer)?);
            },
            e @ b"text" => {
                let text = self.xml.read_text(b"text", &mut buffer)?;
                if let Some(_) = kinetics.text.replace(text) {
                    panic!("ERR: duplicate `text` in `kinetics`");
                }
            }
        }

        Ok(kinetics)
    }

    fn extract_accession<'a, 'b>(&'a mut self, b: &BytesStart<'b>) -> Result<String, XmlError> {
        debug_assert_eq!(b.local_name(), b"accession");
        self.xml.read_text(b"accession", &mut self.buffer)
    }

    fn extract_alternative_products_event(&mut self, b: &BytesStart) -> Result<AlternativeProductEvent, XmlError> {
        debug_assert_eq!(b.local_name(), b"event");

        use self::AlternativeProductEvent::*;

        match b.attributes()
            .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"type").unwrap_or_default())
            .transpose()?
            .as_ref()
            .map(|a| &*a.value)
        {
            Some(b"alternative splicing") => Ok(AlternativeSplicing),
            Some(b"alternative initiation") => Ok(AlternativeInitiation),
            Some(b"alternative promoter") => Ok(AlternativePromoter),
            Some(b"ribosomal frameshifting") => Ok(RibosomalFrameshifting),
            Some(other) => panic!("ERR: invalid `type` in `event`: {:?}", other),
            None => panic!("ERR: missing required `type` in `event`"),
        }
    }

    fn extract_alternative_products_isoform(&mut self, b: &BytesStart) -> Result<Isoform, XmlError> {
        let mut ids = Vec::new();
        let mut names = Vec::new();
        let mut texts = Vec::new();
        let mut optseq: Option<IsoformSequence> = None;

        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"id" => {
                ids.push(self.xml.read_text(b"id", &mut buffer)?);
            },
            e @ b"name" => {
                names.push(self.xml.read_text(b"name", &mut buffer)?);
            },
            e @ b"text" => {
                texts.push(self.xml.read_text(b"text", &mut buffer)?);
            },
            e @ b"sequence" => {
                let seq = self.extract_alternative_products_isoform_sequence(e)?;
                if let Some(_) = optseq.replace(seq) {
                    panic!("ERR: duplicate `sequence` found in `isoform`");
                }
            }
        }

        let seq = optseq
            .expect("ERR: missing required `sequence` element in `isoform`");
        let mut isoform = Isoform::new(seq);
        isoform.names = names;
        isoform.ids = ids;
        isoform.texts = texts;

        Ok(isoform)
    }

    fn extract_alternative_products_isoform_sequence(&mut self, b: &BytesStart) -> Result<IsoformSequence, XmlError> {
        debug_assert_eq!(b.local_name(), b"sequence");

        use self::IsoformSequenceType::*;

        let attr = self.make_attrs(b)?;
        let mut seq = match attr.get(&b"type"[..]).map(|x| &*x.value) {
            Some(b"not described") => IsoformSequence::new(NotDescribed),
            Some(b"described") => IsoformSequence::new(Described),
            Some(b"displayed") => IsoformSequence::new(Displayed),
            Some(b"external") => IsoformSequence::new(External),
            Some(other) => panic!("ERR: invalid value for `type` in `sequence`: {:?}", other),
            None => panic!("ERR: missing `type` attribute in `sequence`"),
        };

        // extract optional reference
        seq.reference = attr.get(&b"ref"[..])
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?;

        // read to end
        self.xml.read_to_end(b.local_name(), &mut Vec::new());
        Ok(seq)
    }

    fn extract_citation(&mut self, b: &BytesStart) -> Result<Citation, XmlError> {
        debug_assert_eq!(b.local_name(), b"citation");

        use self::CitationType::*;

        // extract attributes
        let mut buffer = Vec::new();
        let attr = b
            .attributes()
            .map(|r| r.map(|a| (a.key, a)))
            .collect::<Result< HashMap<_, _>, _>>()?;

        // create citation with proper type
        let mut citation = match &*attr.get(&b"type"[..])
            .expect("ERR: cannot find required `type` in `citation`")
            .value
        {
            b"book" => Citation::new(Book),
            b"journal article" => Citation::new(JournalArticle),
            b"online journal article" => Citation::new(OnlineJournalArticle),
            b"patent" => Citation::new(Patent),
            b"submission" => Citation::new(Submission),
            b"thesis" => Citation::new(Thesis),
            b"unpublished observations" => Citation::new(UnpublishedObservations),
            other => panic!("ERR: invalid `type` in `citation`: {:?}", other),
        };

        // update attributes on citation (TODO)
        // citation.date = attr.get(&b"date"[..])
        //     .map(|v| v.unescape_and_decode_value(&mut self.xml))
        //     .transpose()?;
        citation.name = attr.get(&b"name"[..])
            .map(|v| v.unescape_and_decode_value(&mut self.xml))
            .transpose()?;

        // update citation with children elements
        state_loop!{self, b, buffer,
            e @ b"title" => {
                citation.titles.push(self.xml.read_text(b"title", &mut buffer)?);
            },
            e @ b"authorList" => {
                let mut buffer = Vec::new();
                state_loop!{self, e, buffer,
                    p @ b"person" => {
                        let p = self.xml.read_text(b"person", &mut buffer)
                            .map(CitationName::Person)?;
                        citation.authors.push(p);
                    },
                    p @ b"consortium" => {
                        let c = self.xml.read_text(b"consortium", &mut buffer)
                            .map(CitationName::Consortium)?;
                        citation.authors.push(c);
                    }
                }
            },
            e @ b"editorList" => {
                let mut buffer = Vec::new();
                state_loop!{self, e, buffer,
                    p @ b"person" => {
                        let p = self.xml.read_text(b"person", &mut buffer)
                            .map(CitationName::Person)?;
                        citation.editors.push(p);
                    },
                    p @ b"consortium" => {
                        let c = self.xml.read_text(b"consortium", &mut buffer)
                            .map(CitationName::Consortium)?;
                        citation.editors.push(c);
                    }
                }
            },
            e @ b"locator" => {
                citation.locators.push(self.xml.read_text(b"locator", &mut buffer)?);
            },
            e @ b"dbReference" => {
                citation.db_references.push(self.extract_db_reference(e)?);
            }
        }

        Ok(citation)
    }

    fn extract_cofactor(&mut self, b: &BytesStart) -> Result<Cofactor, XmlError> {
        debug_assert_eq!(b"cofactor", b.local_name());

        let attr = self.make_attrs(b)?;
        let mut optname = None;
        let mut optdbref = None;

        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"name" => {
                let name = self.xml.read_text(b"name", &mut buffer)?;
                if let Some(_) = optname.replace(name) {
                    panic!("ERR: duplicate `name` element in `cofactor`");
                }
            },
            e @ b"dbReference" => {
                let dbref = self.extract_db_reference(e)?;
                if let Some(_) = optdbref.replace(dbref) {
                    panic!("ERR: duplicate `dbReference` in `cofactor`")
                }
            }
        }

        let name = optname.expect("ERR: missing required `name` in `cofactor`");
        let db_reference = optdbref.expect("ERR: missing required `dbReference` in `cofactor`");
        let evidences = self.get_evidences(&attr)?;

        Ok(Cofactor { name, db_reference, evidences })
    }

    fn extract_comment(&mut self, b: &BytesStart) -> Result<Comment, XmlError> {
        macro_rules! comment_loop {
            (
                $self:ident,
                $start:ident,
                $buffer:expr,
                $comment:expr
                $(, $e:ident @ $l:expr => $r:expr )*
            ) => ({
                state_loop!{$self, $start, $buffer,
                    $( $e @ $l => $r ,)*
                    e @ b"text" => {
                        $comment.text.push($self.xml.read_text(b"text", &mut $buffer)?);
                    },
                    e @ b"molecule" => {
                        $comment.molecule = Some(self.extract_molecule(e)?);
                    }
                }
            })
        }

        let attr = self.make_attrs(b)?;
        let mut buffer = Vec::new();
        let mut comment = Comment::new(CommentType::Miscellaneous);

        match attr.get(&b"type"[..]).map(|x| &*x.value) {
            Some(b"function") => {
                comment.ty = CommentType::Function;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"similarity") => {
                comment.ty = CommentType::Similarity;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"subunit") => {
                comment.ty = CommentType::Subunit;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"PTM") => {
                comment.ty = CommentType::Ptm;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"developmental stage") => {
                comment.ty = CommentType::DevelopmentalStage;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"disruption phenotype") => {
                comment.ty = CommentType::DisruptionPhenotype;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"tissue specificity") => {
                comment.ty = CommentType::TissueSpecificity;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"miscellaneous") => {
                comment.ty = CommentType::Miscellaneous;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"induction") => {
                comment.ty = CommentType::Induction;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"caution") => {
                comment.ty = CommentType::Caution;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"pathway") => {
                comment.ty = CommentType::Pathway;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"toxic dose") => {
                comment.ty = CommentType::ToxicDose;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"activity regulation") => {
                comment.ty = CommentType::ActivityRegulation;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"domain") => {
                comment.ty = CommentType::Domain;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"biotechnology") => {
                comment.ty = CommentType::Biotechnology;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"polymorphism") => {
                comment.ty = CommentType::Polymorphism;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"pharmaceutical") => {
                comment.ty = CommentType::Pharmaceutical;
                comment_loop!{self, b, buffer, comment}
            }
            Some(b"allergen") => {
                comment.ty = CommentType::Allergen;
                comment_loop!{self, b, buffer, comment}
            }

            Some(b"subcellular location") => {
                let mut locations = Vec::new();
                comment_loop!{self, b, buffer, comment,
                    e @ b"subcellularLocation" => {
                        locations.push(self.extract_subcellular_location(e)?);
                    }
                }
                comment.ty = CommentType::SubcellularLocation(locations);
            }

            Some(b"alternative products") => {
                let mut product = AlternativeProduct::default();
                comment_loop!{self, b, buffer, comment,
                    e @ b"event" => {
                        product.events.push(self.extract_alternative_products_event(e)?);
                    },
                    e @ b"isoform" => {
                        product.isoforms.push(self.extract_alternative_products_isoform(e)?);
                    }
                }
                comment.ty = CommentType::AlternativeProduct(product);
            }

            Some(b"interaction") => {
                let mut organisms_differ = false;
                let mut experiments = None;
                let mut interactants = Vec::new();

                // extract interaction elements
                comment_loop!{self, b, buffer, comment,
                    e @ b"interactant" => {
                        interactants.push(self.extract_interactant(e)?);
                    },
                    e @ b"organismsDiffer" => {
                        let text = self.xml.read_text(b"organismsDiffer", &mut buffer)?;
                        organisms_differ = bool::from_str(&text)
                            .expect("ERR: could not parse `organismsDiffer` as bool");
                    },
                    e @ b"experiments" => {
                        let text = self.xml.read_text(b"experiments", &mut buffer)?;
                        experiments = usize::from_str(&text)
                            .map(Some)
                            .expect("ERR: could not parse `experiments` as usize");
                    }
                }

                // check that we have 2 interactants
                let i2 = interactants.pop().expect("ERR: missing `interactant` in `interaction`");
                let i1 = interactants.pop().expect("ERR: missing `interactant` in `interaction`");
                if !interactants.is_empty() {
                    panic!("ERR: too many `interactant` in `interaction`");
                }

                // create new interaction
                comment.ty = CommentType::Interaction(Interaction {
                    organisms_differ,
                    experiments: experiments
                        .expect("ERR: missing `experiments` in `interaction`"),
                    interactants: (i1, i2),
                });
            }

            Some(b"sequence caution") => {
                let mut optconflict = None;

                // extract inner `conflict`
                comment_loop!{self, b, buffer, comment,
                    e @ b"conflict" => {
                        let conflict = self.extract_conflict(e)?;
                        if let Some(_) = optconflict.replace(conflict) {
                            panic!("ERR: duplicate `conflict` in `sequence caution`")
                        }
                    }
                }

                // check a `conflict` was extracted
                comment.ty = optconflict.map(CommentType::SequenceCaution)
                    .expect("ERR: missing `conflict` in `sequence caution`");
            }

            Some(b"mass spectrometry") => {
                let mut ms = MassSpectrometry::default();
                ms.mass = attr.get(&b"mass"[..])
                    .map(|x| x.unescape_and_decode_value(&mut self.xml))
                    .transpose()?
                    .map(|s| f64::from_str(&s))
                    .transpose()
                    .expect("could not parse `mass` as f64");
                ms.error = attr.get(&b"error"[..])
                    .map(|x| x.unescape_and_decode_value(&mut self.xml))
                    .transpose()?;
                ms.method = attr.get(&b"method"[..])
                    .map(|x| x.unescape_and_decode_value(&mut self.xml))
                    .transpose()?;

                self.xml.read_to_end(b"comment", &mut buffer)?;
                comment.ty = CommentType::MassSpectrometry(ms);
            }

            Some(b"disease") => {
                let mut optdisease = None;
                comment_loop!{self, b, buffer, comment,
                    e @ b"disease" => {
                        let disease = self.extract_disease(e)?;
                        if let Some(_) = optdisease.replace(disease) {
                            panic!("ERR: duplicate `disease` in `comment`")
                        }
                    }
                }
                comment.ty = CommentType::Disease(optdisease);
            }

            Some(b"biophysicochemical properties") => {
                let mut bcp = BiophysicochemicalProperties::default();
                comment_loop!{self, b, buffer, comment,
                    e @ b"absorption" => {
                        let absorption = self.extract_absorption(e)?;
                        if let Some(_) = bcp.absorption.replace(absorption) {
                            panic!("ERR: duplicate `absorption` in `comment`")
                        }
                    },
                    e @ b"kinetics" => {
                        let kinetics = self.extract_kinetics(e)?;
                        if let Some(_) = bcp.kinetics.replace(kinetics) {
                            panic!("ERR: duplicate `kinetics` in `comment`")
                        }
                    },
                    e @ b"phDependence" => {
                        state_loop!{self, e, self.buffer,
                            t @ b"text" => {
                                let text = self.xml.read_text(b"text", &mut self.buffer)?;
                                bcp.ph_dependence = Some(text);
                            }
                        }
                    },
                    e @ b"redoxPotential" => {
                        state_loop!{self, e, self.buffer,
                            t @ b"text" => {
                                let text = self.xml.read_text(b"text", &mut self.buffer)?;
                                bcp.redox_potential = Some(text);
                            }
                        }
                    },
                    e @ b"temperatureDependence" => {
                        state_loop!{self, e, self.buffer,
                            t @ b"text" => {
                                let text = self.xml.read_text(b"text", &mut self.buffer)?;
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

                comment_loop!{self, b, buffer, comment,
                    e @ b"reaction" => {
                        let reaction = self.extract_reaction(e)?;
                        if let Some(_) = optreact.replace(reaction) {
                            panic!("ERR: duplicate `reaction` in `comment`")
                        }
                    },
                    e @ b"physiologicalReaction" => {
                        physio.push(self.extract_physiological_reaction(e)?);

                    }
                }

                let mut act = optreact.map(CatalyticActivity::new)
                    .expect("ERR: could not find required `reaction` in `comment`");

                if physio.len() > 2 {
                    panic!("ERR: too many `physiologicalReaction` found in `comment`")
                }
                act.physiological_reactions = physio;
                comment.ty = CommentType::CatalyticActivity(act);
            }

            Some(b"online information") => {
                let mut info = OnlineInformation::default();
                info.name = attr.get(&b"name"[..])
                    .map(|a| a.unescape_and_decode_value(&self.xml))
                    .transpose()?;

                comment_loop!{self, b, buffer, comment,
                    e @ b"link" => {
                        let uri = e.attributes()
                            .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"uri").unwrap_or_default())
                            .transpose()?
                            .map(|a| a.unescape_and_decode_value(&mut self.xml))
                            .transpose()?
                            .map(|s| url::Url::parse(&s))
                            .expect("ERR: could not find required `uri` on `link`")
                            .expect("ERR: could not parse `uri` as url::Url");
                        info.links.push(uri);
                        self.xml.read_to_end(b"link", &mut buffer)?;
                    }
                }

                comment.ty = CommentType::OnlineInformation(info);
            }

            Some(b"cofactor") => {
                let mut cofactors = Vec::new();
                comment_loop!{self, b, buffer, comment,
                    e @ b"cofactor" => cofactors.push(self.extract_cofactor(e)?)
                }
                comment.ty = CommentType::Cofactor(cofactors)
            }

            Some(b"RNA editing") => {
                let mut locations = Vec::new();
                comment_loop!{self, b, buffer, comment,
                    e @ b"location" => locations.push(self.extract_feature_location(e)?)
                }
                comment.ty = CommentType::RnaEditing(locations);
            }

            Some(other) => panic!("unknown `type` in `comment`: {:?}", String::from_utf8_lossy(other)),
            None => panic!("could not find required `type` attribute on `comment`"),
        }

        Ok(comment)
    }

    fn extract_conflict(&mut self, b: &BytesStart) -> Result<Conflict, XmlError> {
        debug_assert_eq!(b.local_name(), b"conflict");

        use self::ConflictType::*;

        let attr = self.make_attrs(b)?;
        let mut conflict = match attr.get(&b"type"[..]).map(|x| &*x.value) {
            Some(b"frameshift") => Conflict::new(Frameshift),
            Some(b"erroneous initiation") => Conflict::new(ErroneousInitiation),
            Some(b"erroneous termination") => Conflict::new(ErroneousTermination),
            Some(b"erroneous gene model prediction") => Conflict::new(ErroneousGeneModelPrediction),
            Some(b"erroneous translation") => Conflict::new(ErroneousTranslation),
            Some(b"miscellaneous discrepancy") => Conflict::new(MiscellaneousDiscrepancy),
            Some(other) => panic!("ERR: invalid `type` in `conflict`: {:?}", other),
            None => panic!("ERR: missing required `type` in `conflict`"),
        };

        // extract optional reference
        conflict.reference = attr.get(&b"ref"[..])
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?;

        // extract `sequence` element
        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"sequence" => {
                let sequence = self.extract_conflict_sequence(e)?;
                if let Some(_) = conflict.sequence.replace(sequence) {
                    panic!("ERR: duplicate `sequence` in `conflict`");
                }
            }
        }

        Ok(conflict)
    }

    fn extract_conflict_sequence(&mut self, b: &BytesStart) -> Result<ConflictSequence, XmlError> {
        debug_assert_eq!(b.local_name(), b"sequence");

        let attr = self.make_attrs(b)?;
        let id = attr.get(&b"id"[..])
            .expect("ERR: could not find required `id` attr on `sequence`")
            .unescape_and_decode_value(&mut self.xml)?;
        let version = attr.get(&b"version"[..])
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|s| usize::from_str(&s))
            .transpose()
            .expect("ERR: could not decode `version` as usize");
        let resource = match attr.get(&b"resource"[..]).map(|a| &*a.value) {
            Some(b"EMBL") => ConflictSequenceResource::Embl,
            Some(b"EMBL-CDS") => ConflictSequenceResource::EmblCds,
            Some(other) => panic!("ERR: invalid `resource` in `sequence`: {:?}", other),
            None => panic!("ERR: missing required `resource` in `sequence`"),
        };

        self.xml.read_to_end(b.local_name(), &mut Vec::new());
        Ok(ConflictSequence::with_version(id, resource, version))
    }

    fn extract_db_reference(&mut self, b: &BytesStart) -> Result<DbReference, XmlError> {
        debug_assert_eq!(b.local_name(), b"dbReference");

        let mut buffer = Vec::new();
        let attr = self.make_attrs(b)?;

        let mut db_reference = DbReference::default();
        db_reference.ty = attr.get(&b"type"[..])
            .expect("ERR: could not find required `type` on `dbReference`")
            .unescape_and_decode_value(&mut self.xml)?;
        db_reference.id = attr.get(&b"id"[..])
            .expect("ERR: could not find required `id` on `dbReference`")
            .unescape_and_decode_value(&mut self.xml)?;
        db_reference.evidences = self.get_evidences(&attr)?;

        state_loop!{self, b, buffer,
            e @ b"property" => {
                db_reference.property.push(self.extract_property(e)?);
            },
            e @ b"molecule" => {
                let molecule = self.extract_molecule(e)?;
                if let Some(_) = db_reference.molecule.replace(molecule) {
                    panic!("ERR: duplicate `molecule` found in `db_reference`");
                }
            }
        }

        Ok(db_reference)
    }

    fn extract_disease(&mut self, b: &BytesStart) -> Result<Disease, XmlError> {
        debug_assert_eq!(b.local_name(), b"disease");

        let mut optname = None;
        let mut optdesc = None;
        let mut optacro = None;
        let mut optdbref = None;

        let id = b.attributes()
            .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"id").unwrap_or_default())
            .expect("ERR: could not find required `id` attr on `disease`")?
            .unescape_and_decode_value(&mut self.xml)?;

        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"name" => {
                let name = self.xml.read_text(b"name", &mut buffer)?;
                if let Some(_) = optname.replace(name) {
                    panic!("ERR: duplicate `name` in `disease`");
                }
            },
            e @ b"acronym" => {
                let acronym = self.xml.read_text(b"acronym", &mut buffer)?;
                if let Some(_) = optacro.replace(acronym) {
                    panic!("ERR: duplicate `acronym` in `disease`");
                }
            },
            e @ b"description" => {
                let description = self.xml.read_text(b"description", &mut buffer)?;
                if let Some(_) = optdesc.replace(description) {
                    panic!("ERR: duplicate `description` in `disease`");
                }
            },
            e @ b"dbReference" => {
                let db_reference = self.extract_db_reference(e)?;
                if let Some(_) = optdbref.replace(db_reference) {
                    panic!("ERR: duplicate `db_reference` in `disease`");
                }
            }
        }

        Ok(Disease {
            id,
            name: optname.expect("ERR: missing `name` in `disease`"),
            description: optdesc.expect("ERR: missing `description` in `disease`"),
            acronym: optacro.expect("ERR: missing `acronym` in `disease`"),
            db_reference: optdbref.expect("ERR: missing `db_reference` in `disease`"),
        })
    }

    fn extract_entry(&mut self, b: &BytesStart) -> Result<Entry, XmlError> {
        debug_assert_eq!(b.local_name(), b"entry");

        let mut buffer = Vec::new();
        let attr = self.make_attrs(b)?;

        let dataset = match attr.get(&b"dataset"[..]).map(|a| &*a.value) {
            Some(b"Swiss-Prot") => Dataset::SwissProt,
            Some(b"TrEMBL") => Dataset::TrEmbl,
            Some(other) => panic!("ERR: invalid value for `dataset` attribute of `entry`"),
            None => panic!("ERR: missing required `dataset` attribute of `entry`"),
        };
        let mut entry = Entry::new(dataset);

        state_loop!{self, b, buffer,
            e @ b"accession" => {
                entry.accessions.push(self.extract_accession(e)?);
            },
            e @ b"name" => entry.names.push(self.extract_name(e)?),
            e @ b"protein" => entry.protein = self.extract_protein(e)?,
            e @ b"gene" => entry.genes.push(self.extract_gene(e)?),
            e @ b"organism" => {
                entry.organism = self.extract_organism(e)?;
            },
            e @ b"organismHost" => {
                entry.organism_hosts.push(self.extract_organism(e)?);
            },
            e @ b"reference" => {
                entry.references.push(self.extract_reference(e)?);
            },
            e @ b"comment" => {
                entry.comments.push(self.extract_comment(e)?);
            },
            e @ b"dbReference" => {
                entry.db_references.push(self.extract_db_reference(e)?);
            },
            e @ b"proteinExistence" => {
                entry.protein_existence = self.extract_protein_existence(e)?;
            },
            e @ b"keyword" => {
                entry.keywords.push(self.extract_keyword(e)?);
            },
            e @ b"feature" => {
                entry.features.push(self.extract_feature(e)?);
            },
            e @ b"evidence" => {
                // println!("TODO `evidence` in `entry`");
                self.xml.read_to_end(b"evidence", &mut buffer)?;
            },
            e @ b"sequence" => {
                entry.sequence = self.extract_sequence(e)?;
            },
            e @ b"geneLocation" => {
                entry.gene_location.push(self.extract_gene_location(e)?);
            }
        }

        Ok(entry)
    }

    fn extract_feature(&mut self, b: &BytesStart) -> Result<Feature, XmlError> {
        debug_assert_eq!(b.local_name(), b"feature");

        use self::FeatureType::*;

        // collect the features
        let mut buffer = Vec::new();
        let attr = self.make_attrs(b)?;

        // extract the location and variants
        let mut variation: Vec<String> = Vec::new();
        let mut original: Option<String> = None;
        let mut optloc: Option<FeatureLocation> = None;
        state_loop!{self, b, buffer,
            e @ b"location" => {
                let loc = self.extract_feature_location(e)?;
                if let Some(_) = optloc.replace(loc) {
                    panic!("ERR: duplicate `location` found in `feature`");
                }
            },
            e @ b"original" => {
                original = self.xml.read_text(b"original", &mut buffer).map(Some)?;
            },
            e @ b"variation" => {
                variation.push(self.xml.read_text(b"variation", &mut buffer)?);
            }
        }

        // assume the location was found and extract the feature type
        let location = optloc
            .expect("ERR: could not find required `location` in `feature`");
        let mut feature = match &*attr.get(&b"type"[..])
            .expect("ERR: could not find required `type` attr from `feature`")
            .value
        {
            b"active site" => Feature::new(ActiveSite, location),
            b"binding site" => Feature::new(BindingSite, location),
            b"calcium-binding region" => Feature::new(CalciumBindingRegion, location),
            b"chain" => Feature::new(Chain, location),
            b"coiled-coil region" => Feature::new(CoiledCoilRegion, location),
            b"compositionally biased region" => Feature::new(CompositionallyBiasedRegion, location),
            b"cross-link" => Feature::new(CrossLink, location),
            b"disulfide bond" => Feature::new(DisulfideBond, location),
            b"DNA-binding region" => Feature::new(DnaBindingRegion, location),
            b"domain" => Feature::new(Domain, location),
            b"glycosylation site" => Feature::new(GlycosylationSite, location),
            b"helix" => Feature::new(Helix, location),
            b"initiator methionine" => Feature::new(InitiatorMethionine, location),
            b"lipid moiety-binding region" => Feature::new(LipidMoietyBindingRegion, location),
            b"metal ion-binding site" => Feature::new(MetalIonBindingSite, location),
            b"modified residue" => Feature::new(ModifiedResidue, location),
            b"mutagenesis site" => Feature::new(MutagenesisSite, location),
            b"non-consecutive residues" => Feature::new(NonConsecutiveResidues, location),
            b"non-terminal residue" => Feature::new(NonTerminalResidue, location),
            b"nucleotide phosphate-binding region" => Feature::new(NucleotidePhosphateBindingRegion, location),
            b"peptide" => Feature::new(Peptide, location),
            b"propeptide" => Feature::new(Propeptide, location),
            b"region of interest" => Feature::new(RegionOfInterest, location),
            b"repeat" => Feature::new(Repeat, location),
            b"non-standard amino acid" => Feature::new(NonStandardAminoAcid, location),
            b"sequence conflict" => Feature::new(SequenceConflict, location),
            b"sequence variant" => Feature::new(SequenceVariant, location),
            b"short sequence motif" => Feature::new(ShortSequenceMotif, location),
            b"signal peptide" => Feature::new(SignalPeptide, location),
            b"site" => Feature::new(Site, location),
            b"splice variant" => Feature::new(Site, location),
            b"strand" => Feature::new(Strand, location),
            b"topological domain" => Feature::new(TopologicalDomain, location),
            b"transit peptide" => Feature::new(TransitPeptide, location),
            b"transmembrane region" => Feature::new(TransmembraneRegion, location),
            b"turn" => Feature::new(Turn, location),
            b"unsure residue" => Feature::new(UnsureResidue, location),
            b"zinc finger region" => Feature::new(ZincFingerRegion, location),
            b"intramembrane region" => Feature::new(IntramembraneRegion, location),
            other => panic!("ERR: invalid `type` value in `feature`: {:?}", other),
        };

        // extract optional attributes
        feature.id = attr.get(&b"id"[..])
            .map(|a| a.unescape_and_decode_value(&mut self.xml))
            .transpose()?;
        feature.description = attr.get(&b"description"[..])
            .map(|a| a.unescape_and_decode_value(&mut self.xml))
            .transpose()?;
        feature.reference = attr.get(&b"ref"[..])
            .map(|a| a.unescape_and_decode_value(&mut self.xml))
            .transpose()?;
        feature.evidences = self.get_evidences(&attr)?;
        feature.original = original;
        feature.variation = variation;

        Ok(feature)
    }

    fn extract_feature_location(&mut self, b: &BytesStart) -> Result<FeatureLocation, XmlError> {
        debug_assert_eq!(b.local_name(), b"location");

        let mut optbegin: Option<FeaturePosition> = None;
        let mut optend: Option<FeaturePosition> = None;
        let mut optposition: Option<FeaturePosition> = None;

        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"begin" => {
                if let Some(_) = optbegin.replace(self.extract_feature_position(e)?) {
                    panic!("ERR: duplicate `begin` found in `location`");
                }
            },
            e @ b"end" => {
                if let Some(_) = optend.replace(self.extract_feature_position(e)?) {
                    panic!("ERR: duplicate `end` found in `location`")
                }
            },
            e @ b"position" => {
                if let Some(_) = optposition.replace(self.extract_feature_position(e)?) {
                    panic!("ERR: duplicate `position` found in `location`")
                }
            }
        }

        if let Some(pos) = optposition {
            if (optbegin.is_some() || optend.is_some()) {
                panic!("ERR: cannot have both `begin` or `end` with `position`");
            }
            Ok(FeatureLocation::Position(pos))
        } else {
            let begin = optbegin.expect("ERR: could not find `begin` in `location`");
            let end = optend.expect("ERR: could not find `end` in `location`");
            Ok(FeatureLocation::Range(begin, end))
        }
    }

    fn extract_feature_position(&mut self, b: &BytesStart) -> Result<FeaturePosition, XmlError> {
        let attr = self.make_attrs(b)?;
        let status = match attr.get(&b"status"[..]).map(|a| &*a.value) {
            Some(b"certain") => FeaturePositionStatus::Certain,
            Some(b"uncertain") => FeaturePositionStatus::Uncertain,
            Some(b"less than") => FeaturePositionStatus::Certain,
            Some(b"greater than") => FeaturePositionStatus::Certain,
            Some(b"unknown") => FeaturePositionStatus::Certain,
            Some(other) => panic!("ERR: invalid `status` for `position`: {:?}", other),
            None => FeaturePositionStatus::default(),
        };
        let evidence = self.get_evidences(&attr)?;
        let pos = attr.get(&b"position"[..])
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|x| usize::from_str(&x).expect("ERR: could not decode `position` as usize"));

        self.xml.read_to_end(b.local_name(), &mut Vec::new())?;
        Ok(FeaturePosition { pos, status, evidence })
    }

    fn extract_gene(&mut self, b: &BytesStart) -> Result<Gene, XmlError> {
        debug_assert_eq!(b.local_name(), b"gene");

        let mut gene = Gene::default();
        let mut buffer = Vec::new();

        state_loop!{self, b, buffer,
            e @ b"name" => {
                gene.names.push(self.extract_gene_name(e)?);
            }
        }

        Ok(gene)
    }

    fn extract_gene_location(&mut self, b: &BytesStart) -> Result<GeneLocation, XmlError> {
        debug_assert_eq!(b.local_name(), b"geneLocation");

        use self::GeneLocationType::*;

        let attr = self.make_attrs(&b)?;
        let mut geneloc = match attr.get(&b"type"[..]).map(|a| &*a.value) {
            Some(b"apicoplast") => GeneLocation::new(Apicoplast),
            Some(b"chloroplast") => GeneLocation::new(Chloroplast),
            Some(b"organellar chromatophore") => GeneLocation::new(OrganellarChromatophore),
            Some(b"cyanelle") => GeneLocation::new(Cyanelle),
            Some(b"hydrogenosome") => GeneLocation::new(Hydrogenosome),
            Some(b"mitochondrion") => GeneLocation::new(Mitochondrion),
            Some(b"non-photosynthetic plastid") => GeneLocation::new(NonPhotosyntheticPlasmid),
            Some(b"nucleomorph") => GeneLocation::new(Nucleomorph),
            Some(b"plasmid") => GeneLocation::new(Plasmid),
            Some(b"plastid") => GeneLocation::new(Plastid),
            Some(other) => panic!("ERR: invalid value for `type` in `geneLocation`: {:?}", other),
            None => panic!("ERR: missing required value `type` in `geneLocation`"),
        };
        geneloc.evidences = self.get_evidences(&attr)?;

        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"name" => {
                geneloc.names.push(self.extract_gene_location_name(e)?);
            }
        }

        Ok(geneloc)
    }

    fn extract_gene_location_name(&mut self, b: &BytesStart) -> Result<GeneLocationName, XmlError> {
        debug_assert_eq!(b.local_name(), b"name");

        let attr = self.make_attrs(&b)?;
        let value = self.xml.read_text(b"name", &mut self.buffer)?;

        let status = match attr.get(&b"status"[..]).map(|a| &*a.value) {
            Some(b"known") => GeneLocationStatus::Known,
            Some(b"unknown") => GeneLocationStatus::Unknown,
            Some(other) => panic!("ERR: invalid `status` in `name` of `geneLocation`: {:?}", other),
            None => GeneLocationStatus::Known,
        };

        Ok(GeneLocationName {value, status})
    }

    fn extract_gene_name(&mut self, b: &BytesStart) -> Result<GeneName, XmlError> {
        debug_assert_eq!(b.local_name(), b"name");

        let attr = self.make_attrs(b)?;
        let evidence = self.get_evidences(&attr)?;
        let ty = match attr.get(&b"type"[..]).map(|a| &*a.value) {
            Some(b"primary") => GeneNameType::Primary,
            Some(b"synonym") => GeneNameType::Synonym,
            Some(b"ordered locus") => GeneNameType::OrderedLocus,
            Some(b"ORF") => GeneNameType::Orf,
            _ => panic!("ERR: invalid or missing value for `type` in `name`"),
        };
        let name = self.xml.read_text(b.local_name(), &mut self.buffer)?;
        Ok(GeneName::new_with_evidence(name, ty, evidence))
    }

    fn extract_interactant(&mut self, b: &BytesStart) -> Result<Interactant, XmlError> {
        debug_assert_eq!(b.local_name(), b"interactant");

        let mut interactant = b.attributes()
            .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"intactId").unwrap_or_default())
            .transpose()?
            .map(|a| a.unescape_and_decode_value(&mut self.xml).map(Interactant::new))
            .expect("ERR: could not find required `intactId` attr on `interactant`")?;

        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"id" => {
                let mut id = self.xml.read_text(b"id", &mut buffer)?;
                if let Some(_) = interactant.id.replace(id) {
                    panic!("ERR: duplicate `id` found in `interactant`");
                }
            },
            e @ b"label" => {
                let mut label = self.xml.read_text(b"label", &mut buffer)?;
                if let Some(_) = interactant.label.replace(label) {
                    panic!("ERR: duplicate `label` found in `interactant`");
                }
            }
        }

        Ok(interactant)
    }

    fn extract_keyword(&mut self, b: &BytesStart) -> Result<Keyword, XmlError> {
        debug_assert_eq!(b.local_name(), b"keyword");

        let mut buffer = Vec::new();
        let mut keyword = Keyword::default();

        let attr = self.make_attrs(b)?;
        keyword.evidence = self.get_evidences(&attr)?;
        keyword.id = attr.get(&b"id"[..])
            .expect("ERR: could not find required `id` on `keyword`")
            .unescape_and_decode_value(&mut self.xml)?;
            keyword.value = self.xml.read_text(b.local_name(), &mut buffer)?;

        Ok(keyword)
    }

    fn extract_molecule(&mut self, b: &BytesStart) -> Result<Molecule, XmlError> {
        debug_assert_eq!(b.local_name(), b"molecule");

        let mut buffer = Vec::new();
        let mut molecule = Molecule::default();

        let text = self.xml.read_text(b.local_name(), &mut buffer)?;
        if !text.is_empty() {
            molecule.text = Some(text);
        }

        molecule.id = b.attributes()
            .find(|x| x.is_err() || x.as_ref().map(|a| a.key == b"id").unwrap_or_default())
            .transpose()?
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?;

        Ok(molecule)
    }

    fn extract_name(&mut self, b: &BytesStart) ->  Result<String, XmlError> {
        debug_assert_eq!(b.local_name(), b"name");
        self.xml.read_text(b"name", &mut self.buffer)
    }

    fn extract_organism(&mut self, b: &BytesStart) -> Result<Organism, XmlError> {
        debug_assert!(b.local_name() == b"organism" || b.local_name() == b"organismHost");

        let mut organism = Organism::default();
        let mut buffer = Vec::new();

        let attr = self.make_attrs(b)?;
        organism.evidences = self.get_evidences(&attr)?;

        state_loop!{self, b, buffer,
            e @ b"name" => {
                organism.names.push(self.extract_organism_name(e)?);
            },
            e @ b"dbReference" => {
                organism.db_references.push(self.extract_db_reference(e)?);
            },
            e @ b"lineage" => {
                organism.lineages.push(self.extract_organism_lineage(e)?);
            }
        }

        Ok(organism)
    }

    fn extract_organism_lineage(&mut self, b: &BytesStart) -> Result<OrganismLineage, XmlError> {
        debug_assert_eq!(b.local_name(), b"lineage");

        let mut lineage = OrganismLineage::default();
        let mut buffer = Vec::new();

        state_loop!{self, b, buffer,
            e @ b"taxon" => {
                lineage.taxons.push(self.xml.read_text(b"taxon", &mut buffer)?);
            }
        }

        Ok(lineage)
    }

    fn extract_organism_name(&mut self, b: &BytesStart) -> Result<OrganismName, XmlError> {
        debug_assert_eq!(b.local_name(), b"name");

        use self::OrganismName::*;

        let attr = self.make_attrs(b)?;
        let value = self.xml.read_text(b.local_name(), &mut Vec::new())?;
        match attr.get(&b"type"[..]).map(|a| &*a.value) {
            Some(b"common") => Ok(Common(value)),
            Some(b"full") => Ok(Full(value)),
            Some(b"scientific") => Ok(Scientific(value)),
            Some(b"synonym") => Ok(Synonym(value)),
            Some(b"abbreviation") => Ok(Abbreviation(value)),
            Some(other) => panic!("ERR: invalid value for organism name type: {:?}", other),
            None => panic!("ERR: missing required value for `name` in `organism`"),
        }
    }

    fn extract_physiological_reaction(&mut self, b: &BytesStart) -> Result<PhysiologicalReaction, XmlError> {
        debug_assert_eq!(b.local_name(), b"physiologicalReaction");

        use self::PhysiologicalReactionDirection::*;

        let attr = self.make_attrs(b)?;
        let evidences = self.get_evidences(&attr)?;
        let direction = match attr.get(&b"direction"[..]).map(|a| &*a.value) {
            Some(b"left-to-right") => LeftToRight,
            Some(b"right-to-left")=> RightToLeft,
            Some(other) => panic!("ERR: invalid `direction` for `physiologicalReaction`: {:?}", other),
            None => panic!("ERR: missing required `direction` for `physiologicalReaction`")
        };

        let mut optdbref = None;
        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"dbReference" => {
                let dbref = self.extract_db_reference(e)?;
                if let Some(_) = optdbref.replace(dbref) {
                    panic!("ERR: duplicate `dbReference` found in `reaction`");
                }
            }
        }

        let db_reference = optdbref
            .expect("ERR: could not find required `dbReference` in `physiologicalReaction`");
        Ok(PhysiologicalReaction { db_reference, direction, evidences })
    }

    fn extract_property(&mut self, b: &BytesStart) -> Result<Property, XmlError> {
        debug_assert_eq!(b.local_name(), b"property");

        let mut buffer = Vec::new();
        self.xml.read_to_end(b.local_name(), &mut buffer)?;

        let attr = self.make_attrs(b)?;
        let ty = attr.get(&b"type"[..])
            .expect("ERR: could not find required `type` on `property` element")
            .unescape_and_decode_value(&mut self.xml)?;
        let value = attr.get(&b"value"[..])
            .expect("ERR: could not find required `value` on `property` element")
            .unescape_and_decode_value(&mut self.xml)?;

        Ok(Property::new(ty, value))
    }

    fn extract_protein(&mut self, b: &BytesStart) -> Result<Protein, XmlError> {
        let mut protein = Protein::default();
        let mut buffer = Vec::new();

        state_loop! {self, b, buffer,
            e @ b"recommendedName" => {
                protein.name.recommended = self.extract_protein_name(e).map(Some)?;
            },
            e @ b"alternativeName" => {
                protein.name.alternative.push(self.extract_protein_name(e)?);
            },
            e @ b"component" => {
                // TODO: proper fix to avoid nested `component` in `component`
                protein.components.push(self.extract_protein(e)?.name);
            },
            e @ b"domain" => {
                // TODO: proper fix to avoid nested `domain` in `component`
                protein.domains.push(self.extract_protein(e)?.name);
            },
            e @ b"allergenName" => {
                let value = self.xml.read_text(b"allergenName", &mut buffer)?;
                if let Some(_) = protein.name.allergen.replace(value) {
                    panic!("ERR: duplicate `allergen` in `protein`");
                }
            },
            e @ b"biotechName" => {
                let value = self.xml.read_text(b"biotechName", &mut buffer)?;
                if let Some(_) = protein.name.biotech.replace(value) {
                    panic!("ERR: duplicate `biotech` in `protein`");
                }
            },
            e @ b"cdAntigenName" => {
                let value = self.xml.read_text(b"cdAntigenName", &mut buffer)?;
                protein.name.cd_antigen.push(value);
            },
            e @ b"innName" => {
                let value = self.xml.read_text(b"innName", &mut buffer)?;
                protein.name.inn.push(value);

            }
        }

        Ok(protein)
    }

    fn extract_protein_existence(&mut self, b: &BytesStart) -> Result<ProteinExistence, XmlError> {
        debug_assert_eq!(b.local_name(), b"proteinExistence");

        use self::ProteinExistence::*;

        self.xml.read_to_end(b.local_name(), &mut Vec::new())?;
        match self.make_attrs(b)?.get(&b"type"[..]).map(|a| &*a.value) {
            Some(b"evidence at protein level") => Ok(ProteinLevelEvidence),
            Some(b"evidence at transcript level") => Ok(TranscriptLevelEvidence),
            Some(b"inferred from homology") => Ok(HomologyInferred),
            Some(b"predicted") => Ok(Predicted),
            Some(b"uncertain") => Ok(Uncertain),
            Some(other) => panic!("ERR: invalid `type` in `proteinExistence`: {:?}", other),
            None => panic!("ERR: could not find required `type` on `proteinExistence`"),
        }
    }

    fn extract_protein_name(&mut self, b: &BytesStart) -> Result<ProteinName, XmlError> {
        let mut group = ProteinName::default();

        state_loop!{self, b, self.buffer,
            e @ b"fullName" => {
                group.full = self.xml.read_text(b"fullName", &mut self.buffer)?;
            },
            e @ b"shortName" => {
                group.short.push(self.xml.read_text(b"shortName", &mut self.buffer)?);
            },
            e @ b"ecNumber" => {
                group.ec_number.push(self.xml.read_text(b"ecNumber", &mut self.buffer)?);
            }
        };

        Ok(group)
    }

    fn extract_reaction(&mut self, b: &BytesStart) -> Result<Reaction, XmlError> {
        debug_assert_eq!(b.local_name(), b"reaction");

        let attr = self.make_attrs(b)?;
        let evidences = self.get_evidences(&attr)?;

        let mut db_references = Vec::new();
        let mut opttext = None;

        let mut buffer = Vec::new();
        state_loop!{self, b, buffer,
            e @ b"text" => {
                let text = self.xml.read_text(b"text", &mut buffer)?;
                if let Some(_) = opttext.replace(text) {
                    panic!("ERR: duplicate `text` found in `reaction`");
                }
            },
            e @ b"dbReference" => {
                db_references.push(self.extract_db_reference(e)?);
            }
        }

        let text = opttext.expect("ERR: could not find required `text` in `reaction`");
        Ok(Reaction { text, db_references, evidences })
    }

    fn extract_reference(&mut self, b: &BytesStart) -> Result<Reference, XmlError> {
        debug_assert_eq!(b.local_name(), b"reference");

        let mut reference = Reference::default();
        let mut buffer = Vec::new();

        let attr = self.make_attrs(b)?;
        reference.key = attr.get(&b"key"[..])
            .map(|a| a.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .transpose()
            .expect("ERR: could not decode key number")
            .expect("ERR: could not get `key` attribute from `reference`");
        reference.evidence = attr.get(&b"evidence"[..])
            .map(|a| a.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|e| e.split(' ').map(usize::from_str).collect::<Result<Vec<_>, _>>())
            .transpose()
            .expect("ERR: could not decode evidence number")
            .unwrap_or_default();


        state_loop!{self, b, buffer,
            e @ b"scope" => {
                reference.scope.push(self.xml.read_text(b"scope", &mut buffer)?);
            },
            e @ b"citation" => {
                reference.citation.push(self.extract_citation(e)?);
            },
            e @ b"source" => {
                reference.sources = self.extract_sources(e)?;
            }
        }

        Ok(reference)
    }

    fn extract_sequence(&mut self, b: &BytesStart) -> Result<Sequence, XmlError> {
        debug_assert_eq!(b.local_name(), b"sequence");

        let attr = self.make_attrs(b)?;
        let length = attr.get(&b"length"[..])
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .expect("ERR: could not find required `length` in `sequence`")
            .expect("ERR: could not parse `length` as usize");
        let mass = attr.get(&b"mass"[..])
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .expect("ERR: could not find required `mass` in `sequence`")
            .expect("ERR: could not parse `mass` as usize");
        let checksum = attr.get(&b"checksum"[..])
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|x| u64::from_str_radix(&x, 16))
            .expect("ERR: could not find required `checksum` in `sequence`")
            .expect("ERR: could not parse `checksum` as hex u64");
        // let modified = TODO
        let version = attr.get(&b"version"[..])
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|x| usize::from_str(&x))
            .expect("ERR: could not find required `version` in `sequence`")
            .expect("ERR: could not parse `version` as usize");
        let precursor = attr.get(&b"precursor"[..])
            .map(|x| x.unescape_and_decode_value(&mut self.xml))
            .transpose()?
            .map(|x| bool::from_str(&x))
            .transpose()
            .expect("ERR: could not parse `precursor` as bool");
        let fragment = match attr.get(&b"fragment"[..]).map(|x| &*x.value) {
            Some(b"single") => Some(FragmentType::Single),
            Some(b"multiple") => Some(FragmentType::Multiple),
            Some(other) => panic!("ERR: invalid `fragment` in `sequence`: {:?}", other),
            None => None,
        };

        // let mut buffer = Vec::with_capacity(length);
        let value = self.xml.read_text(b"sequence", &mut self.buffer)?;

        Ok(Sequence {
            value,
            length,
            mass,
            checksum,
            version,
            precursor,
            fragment,
        })
    }

    fn extract_sources(&mut self, b: &BytesStart) -> Result<Vec<Source>, XmlError> {
        debug_assert_eq!(b.local_name(), b"source");

        let mut sources = Vec::new();
        let mut buffer = Vec::new();

        state_loop!{self, b, buffer,
            e @ b"strain" => {
                let evidences = self.make_attrs(e).and_then(|a| self.get_evidences(&a))?;
                let value = self.xml.read_text(b"strain", &mut buffer)?;
                sources.push(Source::Strain { evidences, value })
            },
            e @ b"plasmid" => {
                let evidences = self.make_attrs(e).and_then(|a| self.get_evidences(&a))?;
                let value = self.xml.read_text(b"plasmid", &mut buffer)?;
                sources.push(Source::Plasmid { evidences, value })
            },
            e @ b"transposon" => {
                let evidences = self.make_attrs(e).and_then(|a| self.get_evidences(&a))?;
                let value = self.xml.read_text(b"transposon", &mut buffer)?;
                sources.push(Source::Transposon { evidences, value })
            },
            e @ b"tissue" => {
                let evidences = self.make_attrs(e).and_then(|a| self.get_evidences(&a))?;
                let value = self.xml.read_text(b"tissue", &mut buffer)?;
                sources.push(Source::Tissue { evidences, value })
            }
        }

        Ok(sources)
    }

    fn extract_subcellular_location(&mut self, b: &BytesStart) -> Result<SubcellularLocation, XmlError> {
        let mut subloc = SubcellularLocation::default();
        let mut buffer = Vec::new();

        state_loop!{self, b, buffer,
            e @ b"location" => {
                subloc.locations.push(self.xml.read_text(b"location", &mut buffer)?);
            },
            e @ b"topology" => {
                subloc.topologies.push(self.xml.read_text(b"topology", &mut buffer)?);
            },
            e @ b"orientation" => {
                subloc.orientations.push(self.xml.read_text(b"orientation", &mut buffer)?);
            }
        }

        if subloc.locations.is_empty() {
            panic!("ERR: missing required `location` in `subcellularLocation`");
        }

        Ok(subloc)
    }
}
*/
