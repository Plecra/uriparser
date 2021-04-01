use uriparser_sys;
use core::ptr;
unsafe fn text_range_as_bytes<'a>(range: &uriparser_sys::UriTextRangeA) -> Option<&'a [u8]> {
    if range.first.is_null() {
        None
    } else {
        Some(core::slice::from_raw_parts(
            range.first as *const _,
            range.afterLast as usize - range.first as usize,
        ))
    }    
}

pub struct Uri<'a> {
    raw: uriparser_sys::UriUriA,
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
    pub fn into_owned(mut self) -> Uri<'static> {
        unsafe {
            uriparser_sys::uriMakeOwnerA(&mut self.raw);
        }
        Uri {
            raw: self.raw,
            marker: core::marker::PhantomData,
        }
    }
    pub fn scheme(&self) -> &str {
        core::str::from_utf8(unsafe { text_range_as_bytes(&self.raw.scheme) }.unwrap()).unwrap()
    }
    pub fn userinfo(&self) -> &str {
        core::str::from_utf8(unsafe { text_range_as_bytes(&self.raw.userInfo) }.unwrap()).unwrap()
    }
    pub fn host(&self) -> &str {
        core::str::from_utf8(unsafe { text_range_as_bytes(&self.raw.hostText) }.unwrap()).unwrap()
    }
    pub fn query(&self) -> &str {
        core::str::from_utf8(unsafe { text_range_as_bytes(&self.raw.query) }.unwrap()).unwrap()
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
