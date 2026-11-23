#[derive(Debug, Clone, PartialEq, Default)]
pub enum Action {
    Shutdown,
    ChangeTitle(String),
    /// Answer to `Command::Search`: whether a match was found (and selected).
    SearchResult(bool),
    #[default]
    Ignore,
}
