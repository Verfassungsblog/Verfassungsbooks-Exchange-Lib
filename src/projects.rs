use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Struct holds all project-level settings
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub struct ProjectSettingsV4 {
    pub toc_enabled: bool,
    pub csl_style: Option<String>,
    pub csl_language_code: Option<String>,
    pub metadata_page_additional_html: Option<String>,
    pub cover_image_path: Option<String>,
    pub backcover_image_path: Option<String>,
}

impl From<ProjectSettingsV3> for ProjectSettingsV4{
    fn from(settings: ProjectSettingsV3) -> Self{
        Self{
            toc_enabled: settings.toc_enabled,
            csl_style: settings.csl_style,
            csl_language_code: settings.csl_language_code,
            metadata_page_additional_html: None,
            cover_image_path: None,
            backcover_image_path: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub struct ProjectSettingsV3 {
    pub toc_enabled: bool,
    pub csl_style: Option<String>,
    pub csl_language_code: Option<String>
}

#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub struct ProjectSettingsV2 {
    pub toc_enabled: bool,
    pub csl_style: Option<String>,
}

/// Struct holds a biography in a specified language for a person
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub struct Biography {
    pub content: String,
    pub lang: Option<Language>,
}

/// Enum to differentiate between all supported languages
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub enum Language{
    DE,
    EN
}

/// Struct holds all data for a person (e.g. author or editor)
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub struct Person {
    #[bincode(with_serde)]
    pub id: Option<uuid::Uuid>,
    pub first_names: Option<String>,
    pub last_names: String,
    pub orcid: Option<Identifier>,
    pub gnd: Option<Identifier>,
    pub bios: Option<Vec<Biography>>,
    pub ror: Option<Identifier>,
}


/// Represents an identifier (e.g. DOI, ISBN, ISSN, URL, URN, ORCID, ROR, ...)
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub struct Identifier{
    #[bincode(with_serde)]
    pub id: Option<uuid::Uuid>,
    pub name: String,
    pub value: String,
    pub identifier_type: IdentifierType,
}

impl Identifier{
    /// Create new identifier
    ///
    /// Arguments
    /// * `identifier_type` - Type of identifier as [`IdentifierType`]
    /// * `value` - Value of identifier as [`String`]
    /// * `name` - Name of identifier as [`Option<String>`] - optional
    ///     if not given, the name of the identifier type is used
    ///
    /// Returns
    /// * `Identifier` - New identifier
    pub fn new(identifier_type: IdentifierType, value: String, name: Option<String>) -> Self{
        // If no name is given, use the name of the identifier type
        let name = match name{
            Some(name) => name,
            None => match &identifier_type{
                IdentifierType::DOI => "DOI".to_string(),
                IdentifierType::ISBN => "ISBN".to_string(),
                IdentifierType::ISSN => "ISSN".to_string(),
                IdentifierType::URL => "URL".to_string(),
                IdentifierType::URN => "URN".to_string(),
                IdentifierType::ORCID => "ORCID".to_string(),
                IdentifierType::ROR => "ROR".to_string(),
                IdentifierType::GND => "GND".to_string(),
                IdentifierType::Other(other) => other.clone(),
            },
        };
        Self{
            id: Some(uuid::Uuid::new_v4()),
            name,
            value,
            identifier_type,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub enum IdentifierType{
    DOI,
    ISBN,
    ISSN,
    URL,
    URN,
    ORCID,
    ROR,
    GND,
    Other(String),
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PreparedProject{
    pub metadata: PreparedMetadata,
    pub settings: Option<ProjectSettingsV4>,
    pub sections: Vec<PreparedSection>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PreparedMetadata{
    /// Book Title
    pub title: String,
    /// Subtitle of the book
    pub subtitle: Option<String>,
    /// List of authors of the book
    pub authors: Vec<Person>,
    /// List of editors
    pub editors: Vec<Person>,
    /// URL to a web version of the book or reference
    pub web_url: Option<String>,
    /// List of identifiers of the book (e.g. ISBNs)
    pub identifiers: Option<Vec<Identifier>>,
    /// Date of publication
    pub published: Option<String>,
    /// Languages of the book
    pub languages: Option<Vec<Language>>,
    /// Number of pages of the book (should be automatically calculated)
    pub number_of_pages: Option<u32>,
    /// Short abstract of the book
    pub short_abstract: Option<String>,
    /// Long abstract of the book
    pub long_abstract: Option<String>,
    /// Keywords of the book
    pub keywords: Option<Vec<Keyword>>,
    /// Dewey Decimal Classification (DDC) classes (subject groups)
    pub ddc: Option<String>,
    /// License of the book
    pub license: Option<PreparedLicense>,
    /// Series the book belongs to
    pub series: Option<String>,
    /// Volume of the book in the series
    pub volume: Option<String>,
    /// Edition of the book
    pub edition: Option<String>,
    /// Publisher of the book
    pub publisher: Option<String>,
}

/// Represents a Keyword, optionally with a GND ID
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub struct Keyword{
    pub title: String,
    pub gnd: Option<Identifier>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PreparedLicense{
    CC0: bool,
    CC_BY_4: bool,
    CC_BY_SA_4: bool,
    CC_BY_ND_4: bool,
    CC_BY_NC_4: bool,
    CC_BY_NC_SA_4: bool,
    CC_BY_NC_ND_4: bool,
    other: String,
}

/// Holds all different (CC) licenses or a custom license
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub enum License{
    CC0,
    CC_BY_4,
    CC_BY_SA_4,
    CC_BY_ND_4,
    CC_BY_NC_4,
    CC_BY_NC_SA_4,
    CC_BY_NC_ND_4,
    Other(String),
}

/// implement from License -> PreparedLicense
impl From<License> for PreparedLicense{
    fn from(license: License) -> Self{
        match license{
            License::CC0 => PreparedLicense{CC0: true, CC_BY_4: false, CC_BY_SA_4: false, CC_BY_ND_4: false, CC_BY_NC_4: false, CC_BY_NC_SA_4: false, CC_BY_NC_ND_4: false, other: String::new()},
            License::CC_BY_4 => PreparedLicense{CC0: false, CC_BY_4: true, CC_BY_SA_4: false, CC_BY_ND_4: false, CC_BY_NC_4: false, CC_BY_NC_SA_4: false, CC_BY_NC_ND_4: false, other: String::new()},
            License::CC_BY_SA_4 => PreparedLicense{CC0: false, CC_BY_4: false, CC_BY_SA_4: true, CC_BY_ND_4: false, CC_BY_NC_4: false, CC_BY_NC_SA_4: false, CC_BY_NC_ND_4: false, other: String::new()},
            License::CC_BY_ND_4 => PreparedLicense{CC0: false, CC_BY_4: false, CC_BY_SA_4: false, CC_BY_ND_4: true, CC_BY_NC_4: false, CC_BY_NC_SA_4: false, CC_BY_NC_ND_4: false, other: String::new()},
            License::CC_BY_NC_4 => PreparedLicense{CC0: false, CC_BY_4: false, CC_BY_SA_4: false, CC_BY_ND_4: false, CC_BY_NC_4: true, CC_BY_NC_SA_4: false, CC_BY_NC_ND_4: false, other: String::new()},
            License::CC_BY_NC_SA_4 => PreparedLicense{CC0: false, CC_BY_4: false, CC_BY_SA_4: false, CC_BY_ND_4: false, CC_BY_NC_4: false, CC_BY_NC_SA_4: true, CC_BY_NC_ND_4: false, other: String::new()},
            License::CC_BY_NC_ND_4 => PreparedLicense{CC0: false, CC_BY_4: false, CC_BY_SA_4: false, CC_BY_ND_4: false, CC_BY_NC_4: false, CC_BY_NC_SA_4: false, CC_BY_NC_ND_4: true, other: String::new()},
            License::Other(other) => PreparedLicense{CC0: false, CC_BY_4: false, CC_BY_SA_4: false, CC_BY_ND_4: false, CC_BY_NC_4: false, CC_BY_NC_SA_4: false, CC_BY_NC_ND_4: false, other},
        }
    }
}

/// Represents a single entry in the Table of Contents
#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct TocEntry{
    pub title: String,
    pub level: u32,
    #[bincode(with_serde)]
    pub id: uuid::Uuid,
    pub children: Vec<TocEntry>
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PreparedSection{
    #[bincode(with_serde)]
    pub id: uuid::Uuid,
    pub sub_sections: Vec<PreparedSection>,
    pub children: Vec<PreparedContentBlock>,
    pub metadata: PreparedSectionMetadata,
    pub visible_in_toc: bool,
    pub endnotes: Vec<PreparedEndnote>
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PreparedEndnote{
    pub num: usize,
    #[bincode(with_serde)]
    pub id: uuid::Uuid,
    pub content: String,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PreparedSectionMetadata{
    pub title: String,
    pub subtitle: Option<String>,
    pub toc_title: Option<String>,
    pub authors: Vec<Person>,
    pub editors: Vec<Person>,
    pub web_url: Option<String>,
    pub identifiers: Vec<Identifier>,
    pub published: Option<String>,
    pub lang: PreparedLanguage,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PreparedLanguage{
    pub de: bool,
    pub en: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PreparedContentBlock{
    pub id: String,
    pub block_type: BlockType,
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone, PartialEq)]
pub enum BlockType{
    Paragraph,
    Heading,
    Raw,
    List,
    Quote,
    Image
}