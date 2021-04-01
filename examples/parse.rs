fn main() {
    let text = "https://www.youtube.com/watch?v=HOJ1NVtlnyQ";
    let uri = {
        let mut nulled = Vec::with_capacity(text.len() + 1);
        nulled.extend_from_slice(text.as_bytes());
        nulled.push(0);
        let uri = uriparser::Uri::parse_null_terminated_slice(&nulled)
            .unwrap()
            .into_owned();
        nulled.iter_mut().for_each(|b| *b = b'0');
        uri
    };
    println!("{}", uri.query());
}
