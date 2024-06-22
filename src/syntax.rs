/// to a specific location within an EPUB document. The `Fragment` includes the main `Path`, which
/// is essential for navigating through the document structure, and optionally a `Range` that
/// specifies a span within the document.
///
/// ## Syntax
///
/// In plain text, the syntax is represented as follows:
///
/// ```plaintext
/// fragment = "epubcfi(", (path, [range]), ")"
/// ````
///
/// ## Components
///
/// - **"epubcfi("**: This string marks the beginning of the CFI fragment.
/// - **path**: The primary navigation `Path` through the document.
/// - **range**: An optional component specifying a start and end path to define a `Range` within the
///   document.
/// - **")"**: This character marks the end of the CFI fragment.
#[derive(Debug, PartialEq)]
pub struct Fragment {
    path: Path,
}

impl Fragment {
    pub fn new(path: Path) -> Self {
        Self { path }
    }
}

/// A `Path` in a CFI is a sequence of `Step`s that navigates through the hierarchical structure of
/// an EPUB document to precisely identify a specific element or location. The path allows
/// navigation through varias levels of the document, such as chapters, sections, paragraphs, or
/// other structural elements.
///
/// ## Syntax
///
/// In plain text, the syntax is represented as follows:
///
/// ```text
/// path = step, local_path;
/// ````
///
/// ## Components
///
/// - **step**: The initial step specifies the starting point in the document hierarchy.
/// - **local_path**: A continuation of the path consisting of additional steps, and optionally
///   including offsets or redirections.
///
/// ## Examples
///
/// - **`/4/2/6`**: This path starts at the fourth child element, then moves to its second child,
///   then to the sixth child of the second child.
/// - **`/4/2:10`**: This path starts at the fourth child element, moves to its second child, and
///   specifies an offset of 10 within this second child.
/// - **`/4/2!/6/3:5`**: This path starts at the fourth child element, moves to its second child,
///   and then redirects to another path starting from its sixth child, finally moving to the third
///   child with an offset of 5.
#[derive(Debug, PartialEq)]
pub struct Path {
    /// The intial step in the path, indicating the starting point.
    pub step: Step,
    pub local_path: LocalPath,
}

impl Path {
    pub fn new(step: Step, local_path: LocalPath) -> Self {
        Self { step, local_path }
    }
}

/// A `Range` in an CFI specifies a span of content within a document, defining a start and end
/// point. This is useful for highlighting or selecting a portion of the text or content. Each end
/// of the range is represented by a [LocalPath], and the two paths are separated by commas.
#[derive(Debug, PartialEq)]
pub struct Range {
    start_point: LocalPath,
    end_point: LocalPath,
}

impl Range {
    pub fn new(start_point: LocalPath, end_point: LocalPath) -> Self {
        Self {
            start_point,
            end_point,
        }
    }
}

/// A `Step` is a fundamental part of the `Path` in a CFI, which navigates through the
/// structural elements of an EPUB document. It represents a move from one structural element
/// to another, such as from one HTML element to another in an EPUB document.
///
/// A `Step` starts with a slash, followed by an `Integer` and an optional `Assertion`.
///
/// ## Syntax
///
/// In plain text, the syntax is represented as follows:
///
/// ```text
/// step = "/", integer, [ "[", assertion, "]" ]
/// ```
///
/// ## Components
///
/// - **"/"**: Indicates the beginning of the step.
/// - **integer**: Represents the index of the child element at the current level of the EPUB
///   content document's hierarchy.
/// - **"[", assertion, "]" (optional)**: Specifies an assertion to validate the correctness of the
///   step, ensuring that the target element matches expected conditions.
///
/// ## Detailed Description
///
/// - **integer**: The index selects the nth child element at the current level.
/// - **assertion**: Assertions are optional checks that provide additional validation by
///   specifying conditions that the target element must meet.
///
/// ## Examples
///
/// - **`/4`**: Selects the fourth child element at the current level.
/// - **`/6[2]`**: Selects the sixth child element and verifies that it matches an additional
///   condition `[2]`.
/// - **`/2[lang=en]`**: Selects the second child element and ensures it has a `lang` attribute
///   with a value of "en".
///
#[derive(Debug, PartialEq)]
pub struct Step {
    pub size: u8,
    pub assertion: Option<Assertion>,
}

impl Step {
    pub fn new(size: u8, assertion: Option<Assertion>) -> Self {
        Self { size, assertion }
    }
}

/// An `Assertion` is part of a `Step` that provides addtional validation to ensure the correctness
/// of the identified target element within the EPUB content. It specifies conditions that the
/// target element must satisfy, which can include attributes, values, and other parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Assertion {
    parameters: Option<Vec<(String, String)>>,
    value: Option<String>,
}

impl Assertion {
    pub fn new(parameters: Option<Vec<(String, String)>>, value: Option<String>) -> Self {
        Self { parameters, value }
    }
}

/// A local path in an EPUB Canonical Fragment Identifier (CFI) specifies a specific location
/// within the document, allowing navigation to an element and optionally refining the position
/// within that element using offsets or redirections. A local path can also be referred to as
/// the "relative path".
///
/// - `/4/2`: Navigates to the fourth child element and then to it's second child.
///
/// - `/4/2:5`: Navigates to the fourth child element and then to it's second child, and specifies
///   an offset of 5 characters within the second child.
///
/// - `/3/2!/5`: Navigates to the third child element, then to its second child, redirects, and
///   moves to the fifth child element after redirection.
///
/// - `/4[lang=en]/2:10`: Navigates to the fourth child element, asserting it has a lang attribute
///   equal to "en", then to its second child, and specifies an offset of 10 characters within the
///   second child.
///
/// - `/6/1@3.5:7.2`: Navigates to the sixth child element, then to its first child, and specifies
///   an offset starting at 3.5 units and ending at 7.2 units within the first child.
///
/// # Components
///
/// [EPUB Canonical Fragment Identifiers 1.1
/// Syntax](https://idpf.org/epub/linking/cfi/epub-cfi.html#epubcfi.ebnf.local_path)
///
/// ```plaintext
/// local_path = { step } , ( redirected_path | [ offset ] ) ;
/// ```
///
/// - [step](Step): A navigation step within the document's hierarchy
/// - [redirected_path](RedirectedPath): A path that includes a redirection to another location
///   within the document.
/// - [offset](Offset): An optional numerical value indicating a precise location within the
///   element (e.g, character offset).
///
/// # Examples
///
/// ```rust
/// ```
#[derive(Debug, PartialEq)]
pub struct LocalPath {
    pub steps: Vec<Step>,
    pub redirected_path: Option<RedirectedPath>,
    pub offset: Option<Option<Offset>>,
}

impl LocalPath {
    pub fn new_with_redirected_path(steps: Vec<Step>, redirected_path: RedirectedPath) -> Self {
        Self {
            steps,
            redirected_path: Some(redirected_path),
            offset: None,
        }
    }

    pub fn new_with_offset(steps: Vec<Step>, offset: Option<Offset>) -> Self {
        Self {
            steps,
            redirected_path: None,
            offset: Some(offset),
        }
    }
}

/// A redirected path in an EPUB Canonical Fragment Identifier (CFI) indicates a change in the
/// navigation context within the document. It allows redirection to another element, either
/// specifying an exact position with an offset or providing a new path to follow after the
/// redirection.
///
/// - `!/4/2`: Redirects to the fourth child element and then moves to its second child.
///
/// - `!/4:15`: Redirects to the fourth child element and specifies an offset of 15 characters
///   within this element.
///
/// - `!/3[lang=fr]/1:10`
///
/// # Components
///
/// [EPUB Canonical Fragment Identifiers 1.1
/// Syntax](https://idpf.org/epub/linking/cfi/epub-cfi.html#epubcfi.ebnf.redirected_path)
///
/// ```plaintext
/// redirected_path = "!" , ( offset | path ) ;
/// ```
/// - `"!"`: Indicates the start of the redirection.
/// - [offset](Offset): Specifies a precise position within the redirected element (e.g. character
///   offset).
/// - [path](Path): Specifies a new sequence of steps to navigate through the document after the
///   redirection.
///
/// # Examples
///
/// ```rust
/// ```
#[derive(Debug, PartialEq)]
pub struct RedirectedPath {
    offset: Box<Option<Offset>>,
    path: Box<Option<Path>>,
}

impl RedirectedPath {
    pub fn new(offset: Box<Option<Offset>>, path: Box<Option<Path>>) -> Self {
        Self { offset, path }
    }
}

/// An `Offset` in a CFI specifies a precise position within a specific element. This allows for
/// fine-grained navigation within the content of an EPUB document. An offset can be indicated
/// using one of three formats: Character/colon (`:`), Spatial/at-sign (`@`), or Temporal/tilde
/// (`~`).
///
/// This enum can contain a [`CharacterOffset`], [`SpatialOffset`], or a [`TemporalOffset`]. See
/// their respective documentation for more details.
#[derive(Debug, PartialEq)]
pub enum Offset {
    /// A character, or colon (":"), offset
    Character(CharacterOffset),
    /// A spatial offset, or at-sign ("@") offset
    Spatial(SpatialOffset),
    /// A temporal offset, or tilde ("~") offset
    Temporal(TemporalOffset),
}

pub trait ToOffset {
    fn to_offset(&self) -> Offset;
}

/// Character offset specifies an offset within an element using a colon, ":".
///
/// ## Syntax
///
/// In plain text, the syntax is represented as follows:
///
/// ```plaintext
/// offset = ( ":" , integer ) , [ "[" , assertion , "]" ] ;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CharacterOffset {
    /// Number of characters from the start of the element.
    pub start_at_point: u32,
    pub assertion: Option<Assertion>,
}

impl CharacterOffset {
    pub fn new(start_at_point: u32, assertion: Option<Assertion>) -> Self {
        Self {
            start_at_point,
            assertion,
        }
    }
}

impl ToOffset for CharacterOffset {
    fn to_offset(&self) -> Offset {
        Offset::Character(self.clone())
    }
}

/// Spatial offset specifies an offset using a staring point and an optional range using an
/// at-sign, "@".  The numbers can be floating point values to provide more precision.
///
/// ## Syntax
///
/// In plain text, the syntax is represented as follows:
///
/// ```plaintext
/// offset = ( "@" , number , ":" , number ) , [ "[" , assertion , "]" ] ;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SpatialOffset {
    pub start_at_point: f32,
    pub end_at_point: Option<f32>,
    pub assertion: Option<Assertion>,
}

impl SpatialOffset {
    pub fn new(
        start_at_point: f32,
        end_at_point: Option<f32>,
        assertion: Option<Assertion>,
    ) -> Self {
        Self {
            start_at_point,
            end_at_point,
            assertion,
        }
    }
}

impl ToOffset for SpatialOffset {
    fn to_offset(&self) -> Offset {
        Offset::Spatial(self.clone())
    }
}

/// Temporal offset specifies a floating point offset, optionally combined with an at-sign range.
///
/// ## Syntax
///
/// In plain text, the syntax is represented as follows:
///
/// ```plaintext
/// offset = ( "~" , number , [ "@" , number , ":" , number ] ) , [ "[" , assertion , "]" ] ;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalOffset {
    /// Number of characters or percentage, context-dependent.
    pub start_at: f32,
    pub spatial_range: Option<(f32, f32)>,
    pub assertion: Option<Assertion>,
}

impl TemporalOffset {
    pub fn new(
        start_at: f32,
        spatial_range: Option<(f32, f32)>,
        assertion: Option<Assertion>,
    ) -> Self {
        Self {
            start_at,
            spatial_range,
            assertion,
        }
    }
}

impl ToOffset for TemporalOffset {
    fn to_offset(&self) -> Offset {
        Offset::Temporal(self.clone())
    }
}
