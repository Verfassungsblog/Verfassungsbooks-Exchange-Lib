use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};


pub mod project_settings {
    use bincode::{Decode, Encode};
    use serde::{Deserialize, Serialize};
    use crate::projects::ProjectSettingsV5;

    #[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq, Default)]
    pub struct ProjectSettingsV4 {
        pub toc_enabled: bool,
        pub csl_style: Option<String>,
        pub csl_language_code: Option<String>,
        pub metadata_page_additional_html: Option<String>,
        pub cover_image_path: Option<String>,
        pub backcover_image_path: Option<String>,
    }

    impl From<ProjectSettingsV4> for ProjectSettingsV5 {
        fn from(settings: ProjectSettingsV4) -> Self{
            Self{
                toc_enabled: settings.toc_enabled,
                csl_style: settings.csl_style,
                csl_language_code: settings.csl_language_code,
                metadata_page_additional_html: settings.metadata_page_additional_html,
                cover_image_path: settings.cover_image_path,
                backcover_image_path: settings.backcover_image_path,
                add_soft_hyphens: true,
            }
        }
    }

    impl From<ProjectSettingsV3> for ProjectSettingsV4 {
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
}

pub mod data_storage{
    use bincode::{Decode, Encode};
    use language::Language;
    use serde::{Deserialize, Serialize};
    use crate::projects::{BiographyV2, Identifier, PersonV2};

    /// Enum to differentiate between all supported languages
    #[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
    pub enum OldLanguage {
        DE,
        EN
    }

    impl Into<Language> for OldLanguage {
        fn into(self) -> Language {
            match self{
                OldLanguage::DE => Language::DeDe,
                OldLanguage::EN => Language::EnUs,
            }
        }
    }

    /// Struct holds a biography in a specified language for a person
    #[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
    pub struct BiographyV1 {
        pub content: String,
        pub lang: Option<OldLanguage>,
    }

    impl Into<BiographyV2> for BiographyV1 {
        fn into(self) -> BiographyV2 {
            BiographyV2{
                content: self.content,
                lang: self.lang.map(|x|x.into()),
            }
        }
    }

    /// Struct holds all data for a person (e.g. author or editor)
    #[derive(Deserialize, Serialize, Debug, Encode, Decode, Clone, PartialEq)]
    pub struct PersonV1 {
        #[bincode(with_serde)]
        pub id: Option<uuid::Uuid>,
        pub first_names: Option<String>,
        pub last_names: String,
        pub orcid: Option<Identifier>,
        pub gnd: Option<Identifier>,
        pub bios: Option<Vec<BiographyV1>>,
        pub ror: Option<Identifier>,
    }

    impl Into<PersonV2> for PersonV1 {
        fn into(self) -> PersonV2 {
            PersonV2 {
                id: self.id,
                first_names: self.first_names,
                last_names: self.last_names,
                orcid: self.orcid,
                gnd: self.gnd,
                bios: self.bios.map(|b| b.into_iter().map(|bio| bio.into()).collect()),
                ror: self.ror,
            }
        }
    }
}