use std::cmp::Ordering;
use std::collections::HashMap;
use bincode::{Decode, Encode};
use chrono::{Datelike, Month, NaiveDate, NaiveDateTime};
use language::Language;
use serde::{Deserialize, Serialize};
use crate::deprecated::projects::data_storage::{BiographyV1, OldLanguage};

/// Struct holds all project-level settings
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq, Default)]
pub struct ProjectSettingsV5 {
    pub toc_enabled: bool,
    pub csl_style: Option<String>,
    pub csl_language_code: Option<String>,
    pub metadata_page_additional_html: Option<String>,
    pub cover_image_path: Option<String>,
    pub backcover_image_path: Option<String>,
    pub add_soft_hyphens: bool,
}

/// `metadata_page_additional_html` has no backing column in `projects` and always round-trips
/// as `None`; `toc_enabled`/`add_soft_hyphens` are nullable columns that default to `false`.
#[cfg(feature = "sqlx")]
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for ProjectSettingsV5 {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        use sqlx::Row;
        Ok(ProjectSettingsV5 {
            toc_enabled: row.try_get::<Option<bool>, _>("toc_enabled")?.unwrap_or(false),
            csl_style: row.try_get("csl_style")?,
            csl_language_code: row.try_get("csl_language_code")?,
            metadata_page_additional_html: None,
            cover_image_path: row.try_get("cover_image_path")?,
            backcover_image_path: row.try_get("backcover_image_path")?,
            add_soft_hyphens: row.try_get::<Option<bool>, _>("add_soft_hyphens")?.unwrap_or(false),
        })
    }
}

pub type Biography = BiographyV2;

/// Struct holds a biography in a specified language for a person
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub struct BiographyV2 {
    pub content: String,
    #[bincode(with_serde)]
    pub lang: Option<language::Language>,
}

/// `content`/`language` are the `biographies` table's only columns; `language` is stored as a
/// non-null tag and parsed back into `Option<Language>` (an unrecognized tag round-trips as
/// `None`, same lossiness the manual mapping this replaces already had).
#[cfg(feature = "sqlx")]
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for BiographyV2 {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        use sqlx::Row;
        let language: String = row.try_get("language")?;
        Ok(BiographyV2 {
            content: row.try_get("content")?,
            lang: language::Language::from_tag(&language),
        })
    }
}

pub type Person = PersonV2;

/// Struct holds all data for a person (e.g. author or editor)
#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
pub struct PersonV2 {
    #[bincode(with_serde)]
    pub id: Option<uuid::Uuid>,
    pub first_names: Option<String>,
    pub last_names: String,
    pub orcid: Option<Identifier>,
    pub gnd: Option<Identifier>,
    pub bios: Option<Vec<Biography>>,
    pub ror: Option<Identifier>,
}

/// The `persons` table only stores each identifier's `.value` (its `id`/`name` are
/// regenerated on every load, same lossiness the manual mapping this replaces already had),
/// and `bios` lives in a separate `biographies` table joined per-row — always `None` here;
/// callers that need biographies fetch them separately and fill the field in afterward.
#[cfg(feature = "sqlx")]
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for PersonV2 {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        use sqlx::Row;
        let orcid: Option<String> = row.try_get("orcid")?;
        let gnd: Option<String> = row.try_get("gnd")?;
        let ror: Option<String> = row.try_get("ror")?;
        Ok(PersonV2 {
            id: Some(row.try_get("id")?),
            first_names: row.try_get("first_names")?,
            last_names: row.try_get("last_names")?,
            orcid: orcid.map(|value| Identifier::new(IdentifierType::ORCID, value, None)),
            gnd: gnd.map(|value| Identifier::new(IdentifierType::GND, value, None)),
            ror: ror.map(|value| Identifier::new(IdentifierType::ROR, value, None)),
            bios: None,
        })
    }
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
    pub settings: Option<ProjectSettingsV5>,
    pub sections: Vec<PreparedSection>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PreparedMetadata{
    /// Book Title
    pub title: String,
    /// Subtitle of the book
    pub subtitle: Option<String>,
    /// List of authors of the book
    pub authors: Vec<PersonOrString>,
    /// List of editors
    pub editors: Vec<PersonOrString>,
    /// URL to a web version of the book or reference
    pub web_url: Option<String>,
    /// List of identifiers of the book (e.g. ISBNs)
    pub identifiers: Option<Vec<Identifier>>,
    /// Date of publication
    pub published: Option<DetailedDate>,
    /// Languages of the book
    #[bincode(with_serde)]
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
    /// Optional custom fields
    pub custom_fields: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct DetailedDate{
    pub year: u32,
    pub month: Option<u32>,
    pub month_leading_zero: Option<String>,
    pub month_name: Option<MonthName>,
    pub day: Option<u32>,
    pub day_leading_zero: Option<String>,
    pub day_weekday: Option<WeekdayName>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct MonthName{
    pub january: bool,
    pub february: bool,
    pub march: bool,
    pub april: bool,
    pub may: bool,
    pub june: bool,
    pub july: bool,
    pub august: bool,
    pub september: bool,
    pub october: bool,
    pub november: bool,
    pub december: bool,
}

impl From<u32> for MonthName{
    fn from(value: u32) -> Self {
        let mut res = MonthName{
            january: false,
            february: false,
            march: false,
            april: false,
            may: false,
            june: false,
            july: false,
            august: false,
            september: false,
            october: false,
            november: false,
            december: false,
        };
        match value{
            1 => res.january = true,
            2 => res.february = true,
            3 => res.march = true,
            4 => res.april = true,
            5 => res.may = true,
            6 => res.june = true,
            7 => res.july = true,
            8 => res.august = true,
            9 => res.september = true,
            10 => res.october = true,
            11 => res.november = true,
            12 => res.december = true,
            _ => (),
        }
        res
    }
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct WeekdayName{
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
}

impl From<chrono::Weekday> for WeekdayName{
    fn from(value: chrono::Weekday) -> Self {
        let mut res = WeekdayName{
            monday: false,
            tuesday: false,
            wednesday: false,
            thursday: false,
            friday: false,
            saturday: false,
            sunday: false,
        };
        match value{
            chrono::Weekday::Mon => res.monday = true,
            chrono::Weekday::Tue => res.tuesday = true,
            chrono::Weekday::Wed => res.wednesday = true,
            chrono::Weekday::Thu => res.thursday = true,
            chrono::Weekday::Fri => res.friday = true,
            chrono::Weekday::Sat => res.saturday = true,
            chrono::Weekday::Sun => res.sunday = true,
        }
        res
    }
}

impl From<chrono::NaiveDate> for DetailedDate{
    fn from(value: NaiveDate) -> Self {

        DetailedDate{
            year: value.year() as u32,
            month: Some(value.month() as u32),
            month_leading_zero: Some(add_leading_zero(value.month() as u32)),
            month_name: Some(value.month().into()),
            day: Some(value.day() as u32),
            day_leading_zero: Some(add_leading_zero(value.day() as u32)),
            day_weekday: Some(value.weekday().into()),
        }
    }
}

fn add_leading_zero(value: u32) -> String{
    if value < 10{
        format!("0{}", value)
    }else{
        value.to_string()
    }
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
    pub toc_title_subtitle_override: Option<String>,
    pub subtitle: Option<String>,
    pub authors: Vec<PersonOrString>,
    pub editors: Vec<PersonOrString>,
    pub web_url: Option<String>,
    pub identifiers: Vec<Identifier>,
    pub published: Option<DetailedDate>,
    #[bincode(with_serde)]
    pub lang: Option<Language>,
}

#[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone)]
pub enum PersonOrString{
    Person(Person),
    NameString(String)
}

impl Eq for PersonOrString {}

impl PartialEq<Self> for PersonOrString {
    fn eq(&self, other: &Self) -> bool {
        match self{
            PersonOrString::Person(person1) => {
                if let PersonOrString::Person(person2) = other{
                    return person1.eq(person2)
                }
            }
            PersonOrString::NameString(ns1) => {
                if let PersonOrString::NameString(ns2) = other{
                    return ns1.eq(ns2)
                }
            }
        }
        false
    }
}

impl PartialOrd<Self> for PersonOrString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PersonOrString{
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_a = match self{
            PersonOrString::Person(person) => &person.last_names,
            PersonOrString::NameString(ns) => ns.split(" ").last().unwrap_or(""),
        };

        let cmp_b = match self{
            PersonOrString::Person(person) => &person.last_names,
            PersonOrString::NameString(ns) => ns.split(" ").last().unwrap_or(""),
        };

        cmp_a.cmp(&cmp_b)
    }
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