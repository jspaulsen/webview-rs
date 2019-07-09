pub enum Content<T: AsRef<str>> {
    Url(T),
    Html(T),
}
