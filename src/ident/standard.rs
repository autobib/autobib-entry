use phf::phf_map;

use super::{EntryType, EntryTypeRef, FieldKey, FieldKeyRef};

/// One of the standard entry types defined in the [BibLaTeX 3.21
/// documentation](https://mirrors.ctan.org/macros/latex/contrib/biblatex/doc/biblatex.pdf).
///
/// Note that this type can be converted into an [`EntryType`] or an [`EntryTypeRef<'static>`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StandardEntryType {
    /// An article in a journal, magazine, newspaper, or other periodical which forms a self-contained
    /// unit with its own title.
    Article,
    /// A single-volume book with one or more authors where the authors share credit for the work as a whole.
    Book,
    /// A multi-volume [`@book`](Self::Book).
    MvBook,
    /// A part of a book which forms a self-contained unit with its own title. Note
    InBook,
    /// This type is similar to [`@inbook`](Self::InBook) but intended for works originally published as a stand-alone book.
    BookInBook,
    /// Supplemental material in a [`@book`](Self::Book).
    SuppBook,
    /// A book-like work without a formal publisher or sponsoring institution.
    Booklet,
    /// A single-volume collection with multiple, self-contained contributions by distinct authors which have their own title.
    Collection,
    /// A multi-volume [`@collection`](Self::Collection).
    MvCollection,
    /// A contribution to a collection which forms a self-contained unit with a distinct author and title.
    InCollection,
    /// Supplemental material in a [`@collection`](Self::Collection)
    SuppCollection,
    /// A data set or a similar collection of (mostly) raw data.
    Dataset,
    /// Technical or other documentation, not necessarily in printed form.
    Manual,
    /// A fallback type for entries which do not fit into any other category.
    #[default]
    Misc,
    /// An online resource.
    Online,
    /// A patent or patent request
    Patent,
    /// A complete issue of a periodical, such as a special issue of a journal.
    Periodical,
    /// Supplemental material in a [`periodical`](Self::Periodical).
    SuppPeriodical,
    /// A single-volume conference proceedings
    Proceedings,
    /// A multi-volume [`@proceedings`](Self::Proceedings) entry.
    MvProceedings,
    /// An article in a conference proceedings
    InProceedings,
    /// A single-volume work of reference such as an encyclopedia or a dictionary.
    Reference,
    /// A multi-volume [`@reference`](Self::Reference) entry.
    MvReference,
    /// An article in a work of reference.
    InReference,
    /// A technical report, research report, or white paper published by a university or some other institution.
    Report,
    /// Computer software.
    Software,
    /// A thesis written for an educational institution to satisfy the requirements for a degree.
    Thesis,
    /// A work with an author and a title which has not been formally published, such as a
    /// manuscript or the script of a talk.
    Unpublished,
    // TODO: add a normalization for these aliases
    /// An alias for [`@inproceedings`](Self::InProceedings).
    Conference,
    /// An alias for [`@online`](Self::Online).
    Electronic,
    /// Similar to [`@thesis`](Self::Thesis) with a special type field.
    MastersThesis,
    /// Similar to [`@thesis`](Self::Thesis) with a special type field.
    PhdThesis,
    /// Similar to [`@report`](Self::Report) with a special type field.
    TechReport,
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
            Self::Conference => "conference",
            Self::Electronic => "electronic",
            Self::MastersThesis => "mastersthesis",
            Self::PhdThesis => "phdthesis",
            Self::TechReport => "techreport",
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
    "conference" => StandardEntryType::Conference,
    "electronic" => StandardEntryType::Electronic,
    "mastersthesis" => StandardEntryType::MastersThesis,
    "phdthesis" => StandardEntryType::PhdThesis,
    "techreport" => StandardEntryType::TechReport,
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
    /// An abstract.
    Abstract,
    /// Miscellaneous bibliographic data to be printed at the end of the entry.
    Addendum,
    /// The author(s) of an afterword to the work.
    Afterword,
    /// This field may be useful when implementing a style for annotated bibliographies.
    Annotation,
    /// The author(s) of annotations to the work.
    Annotator,
    /// The authors of the [`title`](Self::Title).
    Author,
    /// The type of author.
    AuthorType,
    /// The author(s) of the [`booktitle`](Self::BookTitle).
    BookAuthor,
    /// If the work is published as part of another one, this is the pagination scheme of the enclosing work.
    BookPagination,
    /// The subtitle related to the [`booktitle`](Self::BookTitle).
    BookSubtitle,
    /// If the [`title`](Self::Title) field indicates the title of a work which is part of a larger publication, the title of the main work is given in this field.
    BookTitle,
    /// An annex to the [`booktitle`](Self::BookTitle),
    BookTitleAddon,
    /// A chapter or section or any other unit of a work.
    Chapter,
    /// The author(s) of a commentary to the work
    Commentator,
    /// The publication date.
    Date,
    /// The Digital Object Identifier of the work.
    Doi,
    /// The edition of a printed publication.
    Edition,
    /// The editor(s) of the [`title`](Self::Title), [`booktitle`](Self::BookTitle), or [`maintitle`](Self::MainTitle), depending on the entry type.
    Editor,
    /// A secondary editor performing a different editorial role.
    EditorA,
    /// Another secondary editor performing a different editorial role.
    EditorB,
    /// Another secondary editor performing a different editorial role.
    EditorC,
    /// The type of editorial role performed by the [`editor`](Self::Editor).
    EditorType,
    /// Like [`editortype`](Self::EditorType) but referring to [`editora`](Self::EditorA).
    EditorAType,
    /// Like [`editortype`](Self::EditorType) but referring to [`editorb`](Self::EditorB).
    EditorBType,
    /// Like [`editortype`](Self::EditorType) but referring to [`editorc`](Self::EditorC).
    EditorCType,
    /// The electronic identifier of an [`@article`](StandardEntryType::Article) or chapter-like section of a larger work often called ‘article number’, ‘paper number’ or the like.
    Eid,
    /// A subtype of an entry type.
    EntrySubType,
    /// The electronic identifier of an online publication.
    EPrint,
    /// Additional information related to the resource indicated by the
    /// [`eprinttype`](Self::EPrintType).
    EPrintClass,
    /// The type of [`eprint`](Self::EPrint) identifier, e. g., the name of the archive, repository, service, or system the [`eprint`](Self::EPrint) field refers to.
    EPrintType,
    /// The date of a conference, a symposium, or some other event in
    /// [`@proceedings`](StandardEntryType::Proceedings) or [`@inproceedings`](StandardEntryType::InProceedings) or
    EventDate,
    /// The title of a conference, a symposium, or some other event in
    /// [`@proceedings`](StandardEntryType::Proceedings) or [`@inproceedings`](StandardEntryType::InProceedings) or
    EventTitle,
    /// An annex to a [`eventtitle`](Self::EventTitle).
    EventTitleAddon,
    /// A local link to a pdf or other version of the work.
    File,
    /// The author(s) of a foreword to the work.
    Foreward,
    /// The holder(s) of a [`@patent`](StandardEntryType::Patent), if different from the
    /// [`author`](Self::Author).
    Holder,
    /// A publication notice for unusual publications which do not fit into any of the common categories.
    HowPublished,
    /// A title to use for indexing instead of the regular [`title`](Self::Title) field.
    IndexTitle,
    /// The name of a university or some other institution, depending on the entry type.
    Institution,
    /// The author(s) of an introduction to the work.
    Introduction,
    /// The International Standard Audiovisual Number of an audiovisual work.
    ISAN,
    /// The International Standard Book Number of a book.
    ISBN,
    /// The International Standard Music Number for printed music such as musical scores.
    ISMN,
    /// The International Standard Technical Report Number of a technical report.
    ISRN,
    /// The International Standard Serial Number of a periodical.
    ISSN,
    /// The issue of a journal.
    Issue,
    /// The subtitle of a specific issue of a journal or other periodical.
    IssueSubtitle,
    /// The title of a specific issue of a journal or other periodical.
    IssueTitle,
    /// An annex to the [`issuetitle`](Self::IssueTitle).
    IssueTitleAddon,
    /// The International Standard Work Code of a musical work.
    ISWC,
    /// The subtitle of a journal, a newspaper, or some other periodical.
    JournalSubtitle,
    /// The name of a journal, a newspaper, or some other periodical.
    JournalTitle,
    /// An annex to the [`journaltitle`](Self::JournalTitle).
    JournalTitleAddon,
    /// A designation to be used by the citation style as a substitute for the regular label if
    /// any data required to generate the regular label is missing.
    Label,
    /// The language(s) of the work.
    Language,
    /// This field may be useful to record information such as a library name and a call number.
    Library,
    /// The place(s) of publication, i.e., the location of the [`publisher`](Self::Publisher) or [`institution`](Self::Institution),
    /// depending on the entry type.
    Location,
    /// The subtitle related to the [`maintitle`](Self::MainTitle).
    MainSubtitle,
    /// The main title of a multi-volume book,
    MainTitle,
    /// An annex to the [`maintitle`](Self::MainTitle).
    MainTitleAddon,
    /// The publication month.
    Month,
    /// An addon to be printed immediately after the author name in the bibliography.
    NameAddon,
    /// Miscellaneous bibliographic data which does not fit into any other field.
    Note,
    /// The number of a journal or the volume/number of a book in a [`series`](Self::Series).
    Number,
    /// The organization(s) that published a [`@manual`](StandardEntryType::Manual) or an [`@online`](StandardEntryType::Online) resource, or sponsored
    /// a conference.
    Organization,
    /// If the work is a translation, a reprint, or something similar, the publication date of
    /// the original edition.
    OrigDate,
    /// If the work is a translation, the language(s) of the original work..
    OrigLanguage,
    /// If the work is a translation, a reprint, or something similar, the
    /// [`location`](Self::Location) of the original edition.
    OrigLocation,
    /// If the work is a translation, a reprint, or something similar, the
    /// [`publisher`](Self::Publisher) of the original edition.
    OrigPublisher,
    /// If the work is a translation, the [`title`](Self::Title) of the original work.
    OrigTitle,
    /// One or more page numbers or page ranges
    Pages,
    /// The total number of pages of the work.
    PageTotal,
    /// The pagination of the work.
    Pagination,
    /// The number of a partial volume, for books only and not journals.
    Part,
    /// The name(s) of the publisher(s).
    Publisher,
    /// The publication state of the work, e. g., ‘in press’.
    PubState,
    /// The title of a reprint of the work.
    ReprintTitle,
    /// The name of a publication series, such as “Studies in …”, or the number of a journal
    /// series.
    Series,
    /// The author(s) of the work, given in an abbreviated form.
    ShortAuthor,
    /// The editor(s) of the work, given in an abbreviated form.
    ShortEditor,
    /// A special designation to be used by the citation style instead of the usual label.
    Shorthand,
    /// An introduction for shorthands in the first citation.
    ShorthandIntro,
    /// A short version or an acronym of the [`journaltitle`](Self::JournalTitle).
    ShortJournal,
    /// A short version or an acronym of the [`series`](Self::Series) field.
    ShortSeries,
    /// The title in an abridged form.
    ShortTitle,
    /// The subtitle of the work.
    Subtitle,
    /// The title of the work.
    Title,
    /// An annex to the [`title`](Self::Title).
    TitleAddon,
    /// The translator(s) of the [`title`](Self::Title) or [`booktitle`](Self::BookTitle), depending on the entry type.
    Translator,
    /// The type of a [`@manual`](StandardEntryType::Manual), patent, report, or thesis.
    Type,
    /// The URL of an online publication.
    Url,
    /// The access date of the address specified in the [`url`](Self::Url) field.
    UrlDate,
    /// The location of a conference, a symposium, or some other event in
    /// [`@proceedings`](StandardEntryType::Proceedings)
    /// and [`@inproceedings`](StandardEntryType::InProceedings) entries.
    Venue,
    /// The revision number of a piece of software, a manual, etc.
    Version,
    /// The volume of a multi-volume book or a periodical.
    Volume,
    /// The total number of volumes of a multi-volume work.
    Volumes,
    /// The year of publication.
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
            Self::ISAN => "isan",
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
            Self::OrigLocation => "origlocation",
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
            Self::ShorthandIntro => "shorthandintro",
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

    /// Whether this is one of the BibTeX-compatible types.
    pub fn is_bibtex(self) -> bool {
        matches!(
            self,
            Self::Address
                | Self::Annote
                | Self::Author
                | Self::BookTitle
                | Self::Chapter
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
    "isan" => StandardFieldKey::ISAN,
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
    "origlocation" => StandardFieldKey::OrigLocation,
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
    "shorthandintro" => StandardFieldKey::ShorthandIntro,
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
