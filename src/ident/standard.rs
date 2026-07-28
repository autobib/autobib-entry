use phf::phf_map;

use super::{EntryType, EntryTypeRef, FieldKey, FieldKeyRef};

/// One of the standard entry types defined in the [BibLaTeX 3.21
/// documentation](https://mirrors.ctan.org/macros/latex/contrib/biblatex/doc/biblatex.pdf).
///
/// Note that this type can be converted into an [`EntryType`] or an [`EntryTypeRef<'static>`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StandardEntryType {
    Article,
    Book,
    MvBook,
    InBook,
    BookInBook,
    SuppBook,
    Booklet,
    Collection,
    MvCollection,
    InCollection,
    SuppCollection,
    Dataset,
    Manual,
    #[default]
    Misc,
    Online,
    Patent,
    Periodical,
    SuppPeriodical,
    Proceedings,
    MvProceedings,
    InProceedings,
    Reference,
    MvReference,
    InReference,
    Report,
    Software,
    Thesis,
    Unpublished,
}

impl StandardEntryType {
    /// Convert this type to the entry type name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Article => "article",
            Self::Book => "book",
            Self::MvBook => "mvbook",
            Self::InBook => "inbook",
            Self::BookInBook => "bookinbook",
            Self::SuppBook => "suppbook",
            Self::Booklet => "booklet",
            Self::Collection => "collection",
            Self::MvCollection => "mvcollection",
            Self::InCollection => "incollection",
            Self::SuppCollection => "suppcollection",
            Self::Dataset => "dataset",
            Self::Manual => "manual",
            Self::Misc => "misc",
            Self::Online => "online",
            Self::Patent => "patent",
            Self::Periodical => "periodical",
            Self::SuppPeriodical => "suppperiodical",
            Self::Proceedings => "proceedings",
            Self::MvProceedings => "mvproceedings",
            Self::InProceedings => "inproceedings",
            Self::Reference => "reference",
            Self::MvReference => "mvreference",
            Self::InReference => "inreference",
            Self::Report => "report",
            Self::Software => "software",
            Self::Thesis => "thesis",
            Self::Unpublished => "unpublished",
        }
    }

    /// Returns if the given name corresponds to an entry type.
    pub fn is_name(s: &str) -> bool {
        STANDARD_ENTRY_TYPE_NAMES.contains_key(s)
    }

    /// Read this type from an entry type name.
    pub fn from_name(s: &str) -> Option<Self> {
        STANDARD_ENTRY_TYPE_NAMES.get(s).cloned()
    }
}

static STANDARD_ENTRY_TYPE_NAMES: phf::Map<&'static str, StandardEntryType> = phf_map! {
    "article" => StandardEntryType::Article,
    "book" => StandardEntryType::Book,
    "mvbook" => StandardEntryType::MvBook,
    "inbook" => StandardEntryType::InBook,
    "bookinbook" => StandardEntryType::BookInBook,
    "suppbook" => StandardEntryType::SuppBook,
    "booklet" => StandardEntryType::Booklet,
    "collection" => StandardEntryType::Collection,
    "mvcollection" => StandardEntryType::MvCollection,
    "incollection" => StandardEntryType::InCollection,
    "suppcollection" => StandardEntryType::SuppCollection,
    "dataset" => StandardEntryType::Dataset,
    "manual" => StandardEntryType::Manual,
    "misc" => StandardEntryType::Misc,
    "online" => StandardEntryType::Online,
    "patent" => StandardEntryType::Patent,
    "periodical" => StandardEntryType::Periodical,
    "suppperiodical" => StandardEntryType::SuppPeriodical,
    "proceedings" => StandardEntryType::Proceedings,
    "mvproceedings" => StandardEntryType::MvProceedings,
    "inproceedings" => StandardEntryType::InProceedings,
    "reference" => StandardEntryType::Reference,
    "mvreference" => StandardEntryType::MvReference,
    "inreference" => StandardEntryType::InReference,
    "report" => StandardEntryType::Report,
    "software" => StandardEntryType::Software,
    "thesis" => StandardEntryType::Thesis,
    "unpublished" => StandardEntryType::Unpublished,
};

impl From<StandardEntryType> for EntryType {
    fn from(value: StandardEntryType) -> Self {
        Self(value.name().into())
    }
}

impl<'a> From<StandardEntryType> for EntryTypeRef<'a> {
    fn from(value: StandardEntryType) -> Self {
        Self(value.name())
    }
}

/// One of the standard field key names defined in the [BibLaTeX 3.21
/// documentation](https://mirrors.ctan.org/macros/latex/contrib/biblatex/doc/biblatex.pdf).
///
/// Note that this type can be converted into a [`FieldKey`] or a [`FieldKeyRef<'static>`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardFieldKey {
    // standard
    Abstract,
    Addendum,
    Afterword,
    Annotation,
    Annotator,
    Author,
    AuthorType,
    BookAuthor,
    BookPagination,
    BookSubtitle,
    BookTitle,
    BookTitleAddon,
    Chapter,
    Commentator,
    Date,
    Doi,
    Edition,
    Editor,
    EditorA,
    EditorB,
    EditorC,
    EditorType,
    EditorAType,
    EditorBType,
    EditorCType,
    Eid,
    EntrySubType,
    EPrint,
    EPrintClass,
    EPrintType,
    EventDate,
    EventTitle,
    EventTitleAddon,
    File,
    Foreward,
    Holder,
    HowPublished,
    IndexTitle,
    Institution,
    Introduction,
    ISBN,
    ISMN,
    ISRN,
    ISSN,
    Issue,
    IssueSubtitle,
    IssueTitle,
    IssueTitleAddon,
    ISWC,
    JournalSubtitle,
    JournalTitle,
    JournalTitleAddon,
    Label,
    Language,
    Library,
    Location,
    MainSubtitle,
    MainTitle,
    MainTitleAddon,
    Month,
    NameAddon,
    Note,
    Number,
    Organization,
    OrigDate,
    OrigLanguage,
    OrigPublisher,
    OrigTitle,
    Pages,
    PageTotal,
    Pagination,
    Part,
    Publisher,
    PubState,
    ReprintTitle,
    Series,
    ShortAuthor,
    ShortEditor,
    Shorthand,
    ShortJournal,
    ShortSeries,
    ShortTitle,
    Subtitle,
    Title,
    TitleAddon,
    Translator,
    Type,
    Url,
    UrlDate,
    Venue,
    Version,
    Volume,
    Volumes,
    Year,

    // aliases
    /// Alias for [`Self::Location`]
    Address,
    /// Alias for [`Self::Annotation`]
    Annote,
    /// Alias for [`Self::EPrintType`]
    ArchivePrefix,
    /// Alias for [`Self::JournalTitle`]
    Journal,
    /// Alias for [`Self::SortKey`]
    Key,
    /// Alias for [`Self::File`]
    Pdf,
    /// Alias for [`Self::EPrintClass`]
    PrimaryClass,
    /// Alias for [`Self::Institution`]
    School,

    // special
    Crossref,
    EntrySet,
    Execute,
    Gender,
    LangId,
    LangIdOpts,
    Ids,
    IndexSortTitle,
    Keywords,
    Options,
    Presort,
    Related,
    RelatedOptions,
    RelatedTypes,
    RelatedString,
    SortKey,
    SortName,
    SortShorthand,
    SortTitle,
    SortYear,
    XData,
    XRef,
}

impl StandardFieldKey {
    /// Convert this type to the field key name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Abstract => "abstract",
            Self::Addendum => "addendum",
            Self::Afterword => "afterword",
            Self::Annotation => "annotation",
            Self::Annotator => "annotator",
            Self::Author => "author",
            Self::AuthorType => "authortype",
            Self::BookAuthor => "bookauthor",
            Self::BookPagination => "bookpagination",
            Self::BookSubtitle => "booksubtitle",
            Self::BookTitle => "booktitle",
            Self::BookTitleAddon => "booktitleaddon",
            Self::Chapter => "chapter",
            Self::Commentator => "commentator",
            Self::Date => "date",
            Self::Doi => "doi",
            Self::Edition => "edition",
            Self::Editor => "editor",
            Self::EditorA => "editora",
            Self::EditorB => "editorb",
            Self::EditorC => "editorc",
            Self::EditorType => "editortype",
            Self::EditorAType => "editoratype",
            Self::EditorBType => "editorbtype",
            Self::EditorCType => "editorctype",
            Self::Eid => "eid",
            Self::EntrySubType => "entrysubtype",
            Self::EPrint => "eprint",
            Self::EPrintClass => "eprintclass",
            Self::EPrintType => "eprinttype",
            Self::EventDate => "eventdate",
            Self::EventTitle => "eventtitle",
            Self::EventTitleAddon => "eventtitleaddon",
            Self::File => "file",
            Self::Foreward => "foreward",
            Self::Holder => "holder",
            Self::HowPublished => "howpublished",
            Self::IndexTitle => "indextitle",
            Self::Institution => "institution",
            Self::Introduction => "introduction",
            Self::ISBN => "isbn",
            Self::ISMN => "ismn",
            Self::ISRN => "isrn",
            Self::ISSN => "issn",
            Self::Issue => "issue",
            Self::IssueSubtitle => "issuesubtitle",
            Self::IssueTitle => "issuetitle",
            Self::IssueTitleAddon => "issuetitleaddon",
            Self::ISWC => "iswc",
            Self::JournalSubtitle => "journalsubtitle",
            Self::JournalTitle => "journaltitle",
            Self::JournalTitleAddon => "journaltitleaddon",
            Self::Label => "label",
            Self::Language => "language",
            Self::Library => "library",
            Self::Location => "location",
            Self::MainSubtitle => "mainsubtitle",
            Self::MainTitle => "maintitle",
            Self::MainTitleAddon => "maintitleaddon",
            Self::Month => "month",
            Self::NameAddon => "nameaddon",
            Self::Note => "note",
            Self::Number => "number",
            Self::Organization => "organization",
            Self::OrigDate => "origdate",
            Self::OrigLanguage => "origlanguage",
            Self::OrigPublisher => "origpublisher",
            Self::OrigTitle => "origtitle",
            Self::Pages => "pages",
            Self::PageTotal => "pagetotal",
            Self::Pagination => "pagination",
            Self::Part => "part",
            Self::Publisher => "publisher",
            Self::PubState => "pubstate",
            Self::ReprintTitle => "reprinttitle",
            Self::Series => "series",
            Self::ShortAuthor => "shortauthor",
            Self::ShortEditor => "shorteditor",
            Self::Shorthand => "shorthand",
            Self::ShortJournal => "shortjournal",
            Self::ShortSeries => "shortseries",
            Self::ShortTitle => "shorttitle",
            Self::Subtitle => "subtitle",
            Self::Title => "title",
            Self::TitleAddon => "titleaddon",
            Self::Translator => "translator",
            Self::Type => "type",
            Self::Url => "url",
            Self::UrlDate => "urldate",
            Self::Venue => "venue",
            Self::Version => "version",
            Self::Volume => "volume",
            Self::Volumes => "volumes",
            Self::Year => "year",
            Self::Address => "address",
            Self::Annote => "annote",
            Self::ArchivePrefix => "archiveprefix",
            Self::Journal => "journal",
            Self::Key => "key",
            Self::Pdf => "pdf",
            Self::PrimaryClass => "primaryclass",
            Self::School => "school",
            Self::Crossref => "crossref",
            Self::EntrySet => "entryset",
            Self::Execute => "execute",
            Self::Gender => "gender",
            Self::LangId => "langid",
            Self::LangIdOpts => "langidopts",
            Self::Ids => "ids",
            Self::IndexSortTitle => "indexsorttitle",
            Self::Keywords => "keywords",
            Self::Options => "options",
            Self::Presort => "presort",
            Self::Related => "related",
            Self::RelatedOptions => "relatedoptions",
            Self::RelatedTypes => "relatedtypes",
            Self::RelatedString => "relatedstring",
            Self::SortKey => "sortkey",
            Self::SortName => "sortname",
            Self::SortShorthand => "sortshorthand",
            Self::SortTitle => "sorttitle",
            Self::SortYear => "sortyear",
            Self::XData => "xdata",
            Self::XRef => "xref",
        }
    }

    /// Returns if the given name corresponds to an entry type.
    pub fn is_name(s: &str) -> bool {
        STANDARD_FIELD_KEY_NAMES.contains_key(s)
    }

    /// Read this type from an entry type name.
    pub fn from_name(s: &str) -> Option<Self> {
        STANDARD_FIELD_KEY_NAMES.get(s).cloned()
    }

    /// Try to convert to a BibTeX-compatible variant.
    pub fn to_bibtex(self) -> Option<Self> {
        match self {
            Self::Location => Some(Self::Address),
            Self::Annotation => Some(Self::Annote),
            Self::JournalTitle => Some(Self::Journal),
            Self::SortKey => Some(Self::Key),
            e if e.is_bibtex() => Some(e),
            _ => None,
        }
    }

    /// Convert an alias variant to the normal type.
    pub fn resolve_alias(self) -> Self {
        match self {
            Self::Address => Self::Location,
            Self::Annote => Self::Annotation,
            Self::ArchivePrefix => Self::EPrintType,
            Self::Journal => Self::JournalTitle,
            Self::Key => Self::SortKey,
            Self::Pdf => Self::File,
            Self::PrimaryClass => Self::EPrintClass,
            Self::School => Self::Institution,
            e => e,
        }
    }

    /// Whether BibLaTeX defines this type to be an alias for another
    pub fn is_alias(self) -> bool {
        matches!(
            self,
            Self::Address
                | Self::Annote
                | Self::ArchivePrefix
                | Self::Journal
                | Self::Key
                | Self::Pdf
                | Self::PrimaryClass
                | Self::School
        )
    }

    /// Whether BibLaTeX defines this type to be a special type.
    pub fn is_special(self) -> bool {
        matches!(
            self,
            Self::Crossref
                | Self::EntrySet
                | Self::Execute
                | Self::Gender
                | Self::LangId
                | Self::LangIdOpts
                | Self::Ids
                | Self::IndexSortTitle
                | Self::Keywords
                | Self::Options
                | Self::Presort
                | Self::Related
                | Self::RelatedOptions
                | Self::RelatedTypes
                | Self::RelatedString
                | Self::SortKey
                | Self::SortName
                | Self::SortShorthand
                | Self::SortTitle
                | Self::SortYear
                | Self::XData
                | Self::XRef
        )
    }

    /// Whether this is one of the BibTeX-compatible types.
    pub fn is_bibtex(self) -> bool {
        matches!(
            self,
            Self::Address
                | Self::Annote
                | Self::Author
                | Self::BookTitle
                | Self::Chapter
                | Self::Crossref
                | Self::Edition
                | Self::Editor
                | Self::HowPublished
                | Self::Institution
                | Self::Journal
                | Self::Key
                | Self::Month
                | Self::Note
                | Self::Number
                | Self::Organization
                | Self::Pages
                | Self::Publisher
                | Self::School
                | Self::Series
                | Self::Title
                | Self::Type
                | Self::Volume
                | Self::Year
        )
    }
}

impl From<StandardFieldKey> for FieldKey {
    fn from(value: StandardFieldKey) -> Self {
        Self(value.name().into())
    }
}

impl<'a> From<StandardFieldKey> for FieldKeyRef<'a> {
    fn from(value: StandardFieldKey) -> Self {
        Self(value.name())
    }
}

static STANDARD_FIELD_KEY_NAMES: phf::Map<&'static str, StandardFieldKey> = phf_map! {
    "abstract" => StandardFieldKey::Abstract,
    "addendum" => StandardFieldKey::Addendum,
    "afterword" => StandardFieldKey::Afterword,
    "annotation" => StandardFieldKey::Annotation,
    "annotator" => StandardFieldKey::Annotator,
    "author" => StandardFieldKey::Author,
    "authortype" => StandardFieldKey::AuthorType,
    "bookauthor" => StandardFieldKey::BookAuthor,
    "bookpagination" => StandardFieldKey::BookPagination,
    "booksubtitle" => StandardFieldKey::BookSubtitle,
    "booktitle" => StandardFieldKey::BookTitle,
    "booktitleaddon" => StandardFieldKey::BookTitleAddon,
    "chapter" => StandardFieldKey::Chapter,
    "commentator" => StandardFieldKey::Commentator,
    "date" => StandardFieldKey::Date,
    "doi" => StandardFieldKey::Doi,
    "edition" => StandardFieldKey::Edition,
    "editor" => StandardFieldKey::Editor,
    "editora" => StandardFieldKey::EditorA,
    "editorb" => StandardFieldKey::EditorB,
    "editorc" => StandardFieldKey::EditorC,
    "editortype" => StandardFieldKey::EditorType,
    "editoratype" => StandardFieldKey::EditorAType,
    "editorbtype" => StandardFieldKey::EditorBType,
    "editorctype" => StandardFieldKey::EditorCType,
    "eid" => StandardFieldKey::Eid,
    "entrysubtype" => StandardFieldKey::EntrySubType,
    "eprint" => StandardFieldKey::EPrint,
    "eprintclass" => StandardFieldKey::EPrintClass,
    "eprinttype" => StandardFieldKey::EPrintType,
    "eventdate" => StandardFieldKey::EventDate,
    "eventtitle" => StandardFieldKey::EventTitle,
    "eventtitleaddon" => StandardFieldKey::EventTitleAddon,
    "file" => StandardFieldKey::File,
    "foreward" => StandardFieldKey::Foreward,
    "holder" => StandardFieldKey::Holder,
    "howpublished" => StandardFieldKey::HowPublished,
    "indextitle" => StandardFieldKey::IndexTitle,
    "institution" => StandardFieldKey::Institution,
    "introduction" => StandardFieldKey::Introduction,
    "isbn" => StandardFieldKey::ISBN,
    "ismn" => StandardFieldKey::ISMN,
    "isrn" => StandardFieldKey::ISRN,
    "issn" => StandardFieldKey::ISSN,
    "issue" => StandardFieldKey::Issue,
    "issuesubtitle" => StandardFieldKey::IssueSubtitle,
    "issuetitle" => StandardFieldKey::IssueTitle,
    "issuetitleaddon" => StandardFieldKey::IssueTitleAddon,
    "iswc" => StandardFieldKey::ISWC,
    "journalsubtitle" => StandardFieldKey::JournalSubtitle,
    "journaltitle" => StandardFieldKey::JournalTitle,
    "journaltitleaddon" => StandardFieldKey::JournalTitleAddon,
    "label" => StandardFieldKey::Label,
    "language" => StandardFieldKey::Language,
    "library" => StandardFieldKey::Library,
    "location" => StandardFieldKey::Location,
    "mainsubtitle" => StandardFieldKey::MainSubtitle,
    "maintitle" => StandardFieldKey::MainTitle,
    "maintitleaddon" => StandardFieldKey::MainTitleAddon,
    "month" => StandardFieldKey::Month,
    "nameaddon" => StandardFieldKey::NameAddon,
    "note" => StandardFieldKey::Note,
    "number" => StandardFieldKey::Number,
    "organization" => StandardFieldKey::Organization,
    "origdate" => StandardFieldKey::OrigDate,
    "origlanguage" => StandardFieldKey::OrigLanguage,
    "origpublisher" => StandardFieldKey::OrigPublisher,
    "origtitle" => StandardFieldKey::OrigTitle,
    "pages" => StandardFieldKey::Pages,
    "pagetotal" => StandardFieldKey::PageTotal,
    "pagination" => StandardFieldKey::Pagination,
    "part" => StandardFieldKey::Part,
    "publisher" => StandardFieldKey::Publisher,
    "pubstate" => StandardFieldKey::PubState,
    "reprinttitle" => StandardFieldKey::ReprintTitle,
    "series" => StandardFieldKey::Series,
    "shortauthor" => StandardFieldKey::ShortAuthor,
    "shorteditor" => StandardFieldKey::ShortEditor,
    "shorthand" => StandardFieldKey::Shorthand,
    "shortjournal" => StandardFieldKey::ShortJournal,
    "shortseries" => StandardFieldKey::ShortSeries,
    "shorttitle" => StandardFieldKey::ShortTitle,
    "subtitle" => StandardFieldKey::Subtitle,
    "title" => StandardFieldKey::Title,
    "titleaddon" => StandardFieldKey::TitleAddon,
    "translator" => StandardFieldKey::Translator,
    "type" => StandardFieldKey::Type,
    "url" => StandardFieldKey::Url,
    "urldate" => StandardFieldKey::UrlDate,
    "venue" => StandardFieldKey::Venue,
    "version" => StandardFieldKey::Version,
    "volume" => StandardFieldKey::Volume,
    "volumes" => StandardFieldKey::Volumes,
    "year" => StandardFieldKey::Year,
    "address" => StandardFieldKey::Address,
    "annote" => StandardFieldKey::Annote,
    "archiveprefix" => StandardFieldKey::ArchivePrefix,
    "journal" => StandardFieldKey::Journal,
    "key" => StandardFieldKey::Key,
    "pdf" => StandardFieldKey::Pdf,
    "primaryclass" => StandardFieldKey::PrimaryClass,
    "school" => StandardFieldKey::School,
    "crossref" => StandardFieldKey::Crossref,
    "entryset" => StandardFieldKey::EntrySet,
    "execute" => StandardFieldKey::Execute,
    "gender" => StandardFieldKey::Gender,
    "langid" => StandardFieldKey::LangId,
    "langidopts" => StandardFieldKey::LangIdOpts,
    "ids" => StandardFieldKey::Ids,
    "indexsorttitle" => StandardFieldKey::IndexSortTitle,
    "keywords" => StandardFieldKey::Keywords,
    "options" => StandardFieldKey::Options,
    "presort" => StandardFieldKey::Presort,
    "related" => StandardFieldKey::Related,
    "relatedoptions" => StandardFieldKey::RelatedOptions,
    "relatedtypes" => StandardFieldKey::RelatedTypes,
    "relatedstring" => StandardFieldKey::RelatedString,
    "sortkey" => StandardFieldKey::SortKey,
    "sortname" => StandardFieldKey::SortName,
    "sortshorthand" => StandardFieldKey::SortShorthand,
    "sorttitle" => StandardFieldKey::SortTitle,
    "sortyear" => StandardFieldKey::SortYear,
    "xdata" => StandardFieldKey::XData,
    "xref" => StandardFieldKey::XRef,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_keys() {
        for (k, v) in STANDARD_FIELD_KEY_NAMES.entries() {
            assert_eq!(&v.name(), k);
            if v.is_bibtex() {
                assert_eq!(v.to_bibtex(), Some(*v));
            }
            if let Some(b) = v.to_bibtex() {
                assert!(b.is_bibtex());
            }
        }
    }

    #[test]
    fn test_entry_types() {
        for (k, v) in STANDARD_ENTRY_TYPE_NAMES.entries() {
            assert_eq!(&v.name(), k);
        }
    }
}
