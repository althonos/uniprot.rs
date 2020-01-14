use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::error::Error;
use crate::parser::FromXml;
use crate::parser::utils::attributes_to_hashmap;
use crate::parser::utils::get_evidences;

use super::feature_location::FeatureLocation;

#[derive(Debug, Clone)]
pub struct Feature {
    // fields
    pub original: Option<String>,
    pub variation: Vec<String>,
    pub location: FeatureLocation,

    // attributes
    pub ty: FeatureType,
    pub id: Option<String>,
    pub description: Option<String>,
    pub evidences: Vec<usize>,
    pub reference: Option<String>,
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
        }
    }
}

impl FromXml for Feature {
    fn from_xml<B: BufRead>(
        event: &BytesStart,
        reader: &mut Reader<B>,
        buffer: &mut Vec<u8>
    ) -> Result<Self, Error> {
        debug_assert_eq!(event.local_name(), b"feature");

        use self::FeatureType::*;

        // collect the attributes
        let attr = attributes_to_hashmap(event)?;

        // extract the location and variants
        let mut variation: Vec<String> = Vec::new();
        let mut original: Option<String> = None;
        let mut optloc: Option<FeatureLocation> = None;
        parse_inner!{event, reader, buffer,
            e @ b"location" => {
                let loc = FeatureLocation::from_xml(&e, reader, buffer)?;
                if let Some(_) = optloc.replace(loc) {
                    return Err(Error::DuplicateElement("location", "feature"));
                }
            },
            b"original" => {
                original = reader.read_text(b"original", buffer).map(Some)?;
            },
            b"variation" => {
                variation.push(reader.read_text(b"variation", buffer)?);
            }
        }

        // assume the location was found and extract the feature type
        let location = optloc
            .ok_or(Error::MissingAttribute("location", "feature"))?;
        let mut feature = match attr.get(&b"type"[..]).map(|a| &*a.value)
            .ok_or(Error::MissingAttribute("type", "feature"))?
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
            .map(|a| a.unescape_and_decode_value(reader))
            .transpose()?;
        feature.description = attr.get(&b"description"[..])
            .map(|a| a.unescape_and_decode_value(reader))
            .transpose()?;
        feature.reference = attr.get(&b"ref"[..])
            .map(|a| a.unescape_and_decode_value(reader))
            .transpose()?;
        feature.evidences = get_evidences(reader, &attr)?;
        feature.original = original;
        feature.variation = variation;

        Ok(feature)
    }
}

#[derive(Debug, Clone)]
pub enum FeatureType {
    ActiveSite,
    BindingSite,
    CalciumBindingRegion,
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
    MetalIonBindingSite,
    ModifiedResidue,
    MutagenesisSite,
    NonConsecutiveResidues,
    NonTerminalResidue,
    NucleotidePhosphateBindingRegion,
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
    IntramembraneRegion
}
