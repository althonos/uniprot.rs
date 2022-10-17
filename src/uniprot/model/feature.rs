use std::io::BufRead;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::Reader;

use crate::error::Error;
use crate::error::InvalidValue;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::decode_attribute;
use crate::parser::utils::get_evidences;
use crate::parser::FromXml;

use super::feature_location::FeatureLocation;
use super::ligand::Ligand;
use super::ligand_part::LigandPart;

#[derive(Debug, Clone)]
/// Describes different types of sequence annotations
pub struct Feature {
    // fields
    /// Describes the original sequence in annotations that describe natural or artifical sequence variations.
    pub original: Option<String>,
    /// Describes the variant sequence in annotations that describe natural or artifical sequence variations.
    pub variation: Vec<String>,
    /// Describes the sequence coordinates of the annotation.
    pub location: FeatureLocation,

    // attributes
    /// Describes the type of a sequence annotation
    pub ty: FeatureType,
    pub id: Option<String>,
    pub description: Option<String>,
    pub evidences: Vec<usize>,
    pub reference: Option<String>,
    pub ligand: Option<Ligand>,
    pub ligand_part: Option<LigandPart>,
}

impl Feature {
    pub fn new(ty: FeatureType, location: FeatureLocation) -> Self {
        Self {
            original: Default::default(),
            variation: Default::default(),
            location,
            ty,
            id: Default::default(),
            description: Default::default(),
            evidences: Default::default(),
            reference: Default::default(),
            ligand: Default::default(),
            ligand_part: Default::default(),
        }
    }
}

impl FromXml for Feature {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"feature");

        use self::FeatureType::*;

        // collect the attributes
        let attr = attributes_to_hashmap(event)?;

        // extract the location and variants
        let mut variation: Vec<String> = Vec::new();
        let mut original: Option<String> = None;
        let mut optloc: Option<FeatureLocation> = None;
        let mut optligand: Option<Ligand> = None;
        let mut optligandpart: Option<LigandPart> = None;
        parse_inner! {event, reader, buffer,
            e @ b"location" => {
                let loc = FeatureLocation::from_xml(&e, reader, buffer)?;
                if optloc.replace(loc).is_some() {
                    return Err(Error::DuplicateElement("location", "feature"));
                }
            },
            b"original" => {
                original = reader.read_text(b"original", buffer).map(Some)?;
            },
            b"variation" => {
                variation.push(reader.read_text(b"variation", buffer)?);
            },
            e @ b"ligand" => {
                let ligand = Ligand::from_xml(&e, reader, buffer)?;
                if optligand.replace(ligand).is_some() {
                    return Err(Error::DuplicateElement("ligand", "feature"));
                }
            },
            e @ b"ligandPart" => {
                let ligandpart = LigandPart::from_xml(&e, reader, buffer)?;
                if optligandpart.replace(ligandpart).is_some() {
                    return Err(Error::DuplicateElement("ligandPart", "feature"));
                }
            }
        }

        // assume the location was found and extract the feature type
        let location = optloc.ok_or(Error::MissingAttribute("location", "feature"))?;

        // create a new Feature with the right `type`
        let mut feature = decode_attribute(event, reader, "type", "feature")
            .map(|ty| Feature::new(ty, location))?;

        // extract optional attributes
        feature.id = attr
            .get(&b"id"[..])
            .map(|a| a.unescape_and_decode_value(reader))
            .transpose()?;
        feature.description = attr
            .get(&b"description"[..])
            .map(|a| a.unescape_and_decode_value(reader))
            .transpose()?;
        feature.reference = attr
            .get(&b"ref"[..])
            .map(|a| a.unescape_and_decode_value(reader))
            .transpose()?;
        feature.evidences = get_evidences(reader, &attr)?;
        feature.original = original;
        feature.variation = variation;
        feature.ligand = optligand;
        feature.ligand_part = optligandpart;

        Ok(feature)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// The type of annotations that can be attached to a sequence.
pub enum FeatureType {
    ActiveSite,
    BindingSite,
    Chain,
    CoiledCoilRegion,
    CompositionallyBiasedRegion,
    CrossLink,
    DisulfideBond,
    DnaBindingRegion,
    Domain,
    GlycosylationSite,
    Helix,
    InitiatorMethionine,
    LipidMoietyBindingRegion,
    ModifiedResidue,
    MutagenesisSite,
    NonConsecutiveResidues,
    NonTerminalResidue,
    Peptide,
    Propeptide,
    RegionOfInterest,
    Repeat,
    NonStandardAminoAcid,
    SequenceConflict,
    SequenceVariant,
    ShortSequenceMotif,
    SignalPeptide,
    Site,
    SpliceVariant,
    Strand,
    TopologicalDomain,
    TransitPeptide,
    TransmembraneRegion,
    Turn,
    UnsureResidue,
    ZincFingerRegion,
    IntramembraneRegion,
}

impl FromStr for FeatureType {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::FeatureType::*;
        match s {
            "active site" => Ok(ActiveSite),
            "binding site" => Ok(BindingSite),
            "chain" => Ok(Chain),
            "coiled-coil region" => Ok(CoiledCoilRegion),
            "compositionally biased region" => Ok(CompositionallyBiasedRegion),
            "cross-link" => Ok(CrossLink),
            "disulfide bond" => Ok(DisulfideBond),
            "DNA-binding region" => Ok(DnaBindingRegion),
            "domain" => Ok(Domain),
            "glycosylation site" => Ok(GlycosylationSite),
            "helix" => Ok(Helix),
            "initiator methionine" => Ok(InitiatorMethionine),
            "lipid moiety-binding region" => Ok(LipidMoietyBindingRegion),
            "modified residue" => Ok(ModifiedResidue),
            "mutagenesis site" => Ok(MutagenesisSite),
            "non-consecutive residues" => Ok(NonConsecutiveResidues),
            "non-terminal residue" => Ok(NonTerminalResidue),
            "peptide" => Ok(Peptide),
            "propeptide" => Ok(Propeptide),
            "region of interest" => Ok(RegionOfInterest),
            "repeat" => Ok(Repeat),
            "non-standard amino acid" => Ok(NonStandardAminoAcid),
            "sequence conflict" => Ok(SequenceConflict),
            "sequence variant" => Ok(SequenceVariant),
            "short sequence motif" => Ok(ShortSequenceMotif),
            "signal peptide" => Ok(SignalPeptide),
            "site" => Ok(Site),
            "splice variant" => Ok(Site),
            "strand" => Ok(Strand),
            "topological domain" => Ok(TopologicalDomain),
            "transit peptide" => Ok(TransitPeptide),
            "transmembrane region" => Ok(TransmembraneRegion),
            "turn" => Ok(Turn),
            "unsure residue" => Ok(UnsureResidue),
            "zinc finger region" => Ok(ZincFingerRegion),
            "intramembrane region" => Ok(IntramembraneRegion),
            other => Err(InvalidValue::from(other)),
        }
    }
}
