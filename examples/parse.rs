fn main() {
    let uri = uriparser::Uri::parse(b"https://www.youtube.com/watch?v=HOJ1NVtlnyQ")
        .unwrap()
        .into_owned();
    println!("{}", uri.query());
}
