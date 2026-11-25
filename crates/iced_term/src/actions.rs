#[derive(Debug, Clone, PartialEq, Default)]
pub enum Action {
    Shutdown,
    ChangeTitle(String),
    /// Answer to `Command::Search`: whether a match was found (and selected).
    SearchResult(bool),
    /// `Ctrl+click` on a link: the embedder decides what opening it means (IDE TEA RFC 0025).
    OpenLink(crate::links::Link),
    #[default]
    Ignore,
}
