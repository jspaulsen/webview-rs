/// Content displayable inside a [`WebView`].
///
/// # Variants
///
/// - `Url` - Content to be fetched from a URL.
/// - `Html` - A string containing literal HTML.
///
/// [`WebView`]: struct.WebView.html
pub enum Content<T: AsRef<str>> {
    Url(T),
    Html(T),
}
