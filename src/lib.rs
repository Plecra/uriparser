//! These APIs are all pretty rough for now - only use them after verifying their safety

use uriparser_sys::{self, UriPathSegmentA, UriTextRangeA, UriUriA, uriToStringA, uriToStringCharsRequiredA};
use core::{fmt, ptr, cmp};
fn bool_to_uri(b: bool) -> uriparser_sys::UriBool {
    if b {
        uriparser_sys::URI_TRUE as i32
    } else {
        uriparser_sys::URI_FALSE as i32
    }
}
pub fn escape<'o>(text: &[u8], out: &'o mut [u8], replace_space: bool, normalize_line_breaks: bool) -> &'o mut [u8] {
    assert!(out.len() >= text.len() * if normalize_line_breaks {
        6
    } else {
        3
    } + 1);
    let end = unsafe {
        uriparser_sys::uriEscapeExA(
            text.as_ptr() as *const _, 
            text.as_ptr().add(text.len()) as *const _,
            out.as_mut_ptr() as *mut _, 
            bool_to_uri(replace_space), 
            bool_to_uri(normalize_line_breaks)
        )
    };
    let new_len = end as usize - out.as_ptr() as usize;
    &mut out[..new_len]
}
pub enum LineBreak {
    Cr,
    Lf,
    Crlf,
}
fn hex(v: u8) -> Option<u8> {
    match v {
        b'a'..=b'f' => Some(v - b'a' + 10),
        b'A'..=b'F' => Some(v - b'A' + 10),
        b'0'..=b'9' => Some(v - b'0'),
        _ => None
    }
}
pub fn unescape(text: &mut [u8], replace_plus: bool, line_breaks: LineBreak) -> &mut [u8] {
    let last_byte = match text {
        [.., p, a, b] if *p == b'%' => {
            if let Some(n) = hex(*a).and_then(|a| hex(*b).map(|b| a * 16 + b)) {
                *p = 0;
                n
            } else {
                core::mem::replace(b, 0)
            }
        }
        [.., l] => core::mem::replace(l, 0),
        [] => return text,
    };
    let last = unsafe {
        uriparser_sys::uriUnescapeInPlaceExA(text.as_mut_ptr() as *mut _, bool_to_uri(replace_plus), match line_breaks {
            LineBreak::Cr => uriparser_sys::UriBreakConversionEnum_URI_BR_TO_CR,
            LineBreak::Lf => uriparser_sys::UriBreakConversionEnum_URI_BR_TO_LF,
            LineBreak::Crlf => uriparser_sys::UriBreakConversionEnum_URI_BR_TO_CRLF,
        })
    };
    unsafe {
        *(last as *const u8 as *mut u8) = last_byte;
    }
    let new_len = last as usize - text.as_ptr() as usize;
    &mut text[..=new_len]
}
unsafe fn text_range_as_bytes<'a>(range: &UriTextRangeA) -> Option<&'a [u8]> {
    if range.first.is_null() {
        None
    } else {
        Some(core::slice::from_raw_parts(
            range.first as *const _,
            range.afterLast as usize - range.first as usize,
        ))
    }    
}
#[derive(Debug)]
pub struct Uri<'a> {
    raw: UriUriA,
    marker: core::marker::PhantomData<&'a [u8]>,
}

#[derive(Debug)]
pub struct ParseError {
    errpos: usize,
}

impl ParseError {
    pub fn pos(&self) -> usize {
        self.errpos
    }
    fn from_code(code: u32, errpos: usize) -> Self {
        match code {
            uriparser_sys::URI_ERROR_SYNTAX => Self { errpos },
            uriparser_sys::URI_ERROR_NULL => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_MALLOC => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_OUTPUT_TOO_LARGE => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_NOT_IMPLEMENTED => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_RANGE_INVALID => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_MEMORY_MANAGER_INCOMPLETE => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_TOSTRING_TOO_LONG => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_ADDBASE_REL_BASE => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_REMOVEBASE_REL_BASE => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_REMOVEBASE_REL_SOURCE => panic!("unexpected error"),
            uriparser_sys::URI_ERROR_MEMORY_MANAGER_FAULTY => panic!("unexpected error"),
            err => unimplemented!("unknown error {}", err),
        }
    }
}

impl<'a> Uri<'a> {
    pub fn parse(uri: &'a [u8]) -> Result<Self, ParseError> {
        let mut raw = Default::default();
        let mut errpos = ptr::null();
        match unsafe { uriparser_sys::uriParseSingleUriExA(&mut raw, uri.as_ptr() as *const _, uri.as_ptr().add(uri.len()) as *const _, &mut errpos) } as u32
        {
            uriparser_sys::URI_SUCCESS => Ok(Self {
                raw,
                marker: core::marker::PhantomData,
            }),
            err => Err(ParseError::from_code(err, errpos as usize - uri.as_ptr() as usize)),
        }
    }
    pub fn resolve(&self, reference: &Uri<'_>, strict: bool) -> Result<Uri<'static>, ()> {
        let mut out = Default::default();
        match unsafe {
            uriparser_sys::uriAddBaseUriExA(&mut out, &reference.raw, &self.raw, if strict {
                uriparser_sys::UriResolutionOptionsEnum_URI_RESOLVE_STRICTLY
            } else {
                uriparser_sys::UriResolutionOptionsEnum_URI_RESOLVE_IDENTICAL_SCHEME_COMPAT
            })
        } as u32 {
            uriparser_sys::URI_SUCCESS => Ok(Uri {
                raw: out,
                marker: core::marker::PhantomData,
            }),
            err => Err(())
        }
    }
    pub fn as_relative(&self, base: &Uri<'_>, from_domain_root: bool) -> Result<Uri<'static>, ()> {
        let mut out = Default::default();
        match unsafe {
            uriparser_sys::uriRemoveBaseUriA(&mut out, &self.raw, &base.raw, bool_to_uri(from_domain_root))
        } as u32 {
            uriparser_sys::URI_SUCCESS => Ok(Uri {
                raw: out,
                marker: core::marker::PhantomData,
            }),
            err => Err(())
        }
    }
    pub fn into_owned(mut self) -> Uri<'static> {
        unsafe {
            uriparser_sys::uriMakeOwnerA(&mut self.raw);
        }
        Uri {
            raw: self.raw,
            marker: core::marker::PhantomData,
        }
    }
    pub fn scheme(&self) -> Option<&str> {
        unsafe { text_range_as_bytes(&self.raw.scheme) }.map(|s| core::str::from_utf8(s).unwrap())
    }
    pub fn userinfo(&self) -> Option<&str> {
        unsafe { text_range_as_bytes(&self.raw.userInfo) }.map(|s| core::str::from_utf8(s).unwrap())
    }
    pub fn host(&self) -> Option<&str> {
        unsafe { text_range_as_bytes(&self.raw.hostText) }.map(|s| core::str::from_utf8(s).unwrap())
    }
    pub fn port(&self) -> Option<&str> {
        unsafe { text_range_as_bytes(&self.raw.portText) }.map(|s| core::str::from_utf8(s).unwrap())
    }
    pub fn path(&self) -> Path<'_> {
        let head = core::ptr::NonNull::new(self.raw.pathHead).map(|ptr| unsafe { &*ptr.as_ptr() });
        let is_absolute = self.raw.absolutePath == uriparser_sys::URI_TRUE as i32 || (self.host().is_some() && head.is_some());
        Path {
            is_absolute,
            head
        }
    }

    pub fn query(&self) -> Option<&str> {
        unsafe { text_range_as_bytes(&self.raw.query) }.map(|s| core::str::from_utf8(s).unwrap())
    }
    pub fn fragment(&self) -> Option<&str> {
        unsafe { text_range_as_bytes(&self.raw.fragment) }.map(|s| core::str::from_utf8(s).unwrap())
    }
}
impl PartialEq<Uri<'_>> for Uri<'_> {
    fn eq(&self, other: &Uri<'_>) -> bool {
        (unsafe {
            uriparser_sys::uriEqualsUriA(&self.raw, &other.raw)
        }) as u32 == uriparser_sys::URI_TRUE
    }
}
impl ToString for Uri<'_> {
    fn to_string(&self) -> String {
        let mut capacity = 0;
        assert_eq!(unsafe {
            uriToStringCharsRequiredA(&self.raw, &mut capacity)
        } as u32, uriparser_sys::URI_SUCCESS);
        let mut s = String::with_capacity(capacity as usize);
        let mut written = 0;
        assert_eq!(unsafe {
            uriToStringA(s.as_bytes_mut().as_mut_ptr() as *mut _, &self.raw, capacity, &mut written)
        } as u32, uriparser_sys::URI_SUCCESS);
        unsafe {
            s.as_mut_vec().set_len(written as usize);
        }
        s
    }
}
pub struct Path<'a> {
    is_absolute: bool,
    head: Option<&'a UriPathSegmentA>,
}
use fmt::Write;
impl fmt::Debug for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        if self.is_absolute {
            f.write_char('/')?;
        }
        let mut segments = self.segments();
        if let Some(segment) = segments.next() {
            write!(f, "{}", segment.escape_debug())?;
            for segment in segments {
                f.write_char('/')?;
                write!(f, "{}", segment.escape_debug())?;
            }
        }
        Ok(())
    }
}
impl cmp::PartialEq<[u8]> for Path<'_> {
    fn eq(&self, mut other: &[u8]) -> bool {
        macro_rules! unwrap {
            ($e:expr) => {match $e {
                Some(v) => v,
                None => return false
            }};
        }
        if self.is_absolute {
            other = unwrap!(other.strip_prefix(b"/"));
        }
        let mut segments = self.segments();
        if let Some(segment) = segments.next() {
            other = unwrap!(other.strip_prefix(segment.as_bytes()));
            for segment in segments {
                other = unwrap!(other.strip_prefix(b"/"));
                other = unwrap!(other.strip_prefix(segment.as_bytes()));
            }
        }
        true
    }
}
impl cmp::PartialEq<str> for Path<'_> {
    fn eq(&self, other: &str) -> bool {
        self.eq(other.as_bytes())
    }
}
impl cmp::PartialEq for Path<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.is_absolute.eq(&other.is_absolute) && self.segments().eq(other.segments())
    }
}
impl fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_absolute {
            f.write_char('/')?;
        }
        let mut segments = self.segments();
        if let Some(segment) = segments.next() {
            f.write_str(segment)?;
            for segment in segments {
                f.write_char('/')?;
                f.write_str(segment)?;
            }
        }
        Ok(())
    }
}
impl<'a> Path<'a> {
    pub fn segments(&self) -> impl Iterator<Item = &'a str> + 'a {
        let mut next = self.head;
        core::iter::from_fn(move || {
            next.map(|segment| {
                next = core::ptr::NonNull::new(segment.next).map(|ptr| unsafe { &*ptr.as_ptr() });
                core::str::from_utf8(unsafe { text_range_as_bytes(&segment.text) }.unwrap()).unwrap()
            })
        })
    }
}
impl Uri<'_> {
    fn into_raw(self) -> uriparser_sys::UriUriA {
        self.raw
    }
    unsafe fn from_raw(raw: uriparser_sys::UriUriA) -> Self {
        Self {
            raw,
            marker: core::marker::PhantomData,
        }
    }
    unsafe fn as_raw(&self) -> &uriparser_sys::UriUriA {
        &self.raw
    }
    unsafe fn as_mut_raw(&mut self) -> &mut uriparser_sys::UriUriA {
        &mut self.raw
    }
}

impl Drop for Uri<'_> {
    fn drop(&mut self) {
        unsafe {
            uriparser_sys::uriFreeUriMembersA(&mut self.raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Uri;

    #[test]
    fn parsing() {
        let uri = Uri::parse(b"https://www.youtube.com/watch?v=HOJ1NVtlnyQ").unwrap();
        assert_eq!(Some("v=HOJ1NVtlnyQ"), uri.query());
        Uri::parse(b"foobar://abc.com/ooh##").expect_err("no hashes in fragment");
    }
}