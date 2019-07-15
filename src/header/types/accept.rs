extern crate lazy_static;
extern crate mime;

use std::ops::{Deref, DerefMut};
use std::iter::{once, FromIterator};
use std::convert::TryFrom;
use std::hash::Hasher;
use std::hash::Hash;
use std::str;
use crate::header::map::TypedHeader;
use linked_hash_set::LinkedHashSet;
use crate::header::value::HeaderValue;
use crate::header::name::HeaderName;
use crate::syntax;
use syntax::is_token;
use std::fmt::{self, Display, Formatter};
use std::string::ToString;
use itertools::Itertools;
use mime::*;




/// The Accept request-header field can be used to specify certain
/// presentation description and parameter media types [RFC6838] that are
/// acceptable for the response to the DESCRIBE request.
/// 
/// ```text
/// Accept            =  "Accept" HCOLON
///                        [ accept-range *(COMMA accept-range) ]
///    accept-range      =  media-type-range [SEMI accept-params]
///    accept-params     =  "q" EQUAL qvalue *(SEMI generic-param )
///    qvalue            =  ( "0" [ "." *3DIGIT ] )
///                      /  ( "1" [ "." *3("0") ] )
///    generic-param   =  token [ EQUAL gen-value ] //no example usage so not included
///    gen-value       =  token / host / quoted-string
///    host              = < As defined in RFC 3986>
///
///    media-type-range  =  ( "*//*"
///                            / ( m-type SLASH "*" )
///                            / ( m-type SLASH m-subtype )
///                                ) *( SEMI m-parameter )
///    m-type             =  discrete-type / composite-type
///    discrete-type      =  "text" / "image" / "audio" / "video"
///                      /  "application" / extension-token
///    composite-type   =  "message" / "multipart" / extension-token
///    extension-token  =  ietf-token / x-token
///    ietf-token       =  token
///    x-token          =  "x-" token
///    m-subtype        =  extension-token / iana-token
///    iana-token       =  token
///    m-parameter      =  m-attribute EQUAL m-value
///    m-attribute      =  token
///    m-value          =  token / quoted-string
///    token           =  1*(%x21 / %x23-27 / %x2A-2B / %x2D-2E / %x30-39
///                        /  %x41-5A / %x5E-7A / %x7C / %x7E) //any CHAR except CTLs or tspecials
/// ```
/// 
/// All values seperated with commas will be converted into [`MediaType`] type.
/// 
/// # Examples
/// 
/// ```
/// use std::convert::TryFrom;
///
/// use rtsp::header::map::TypedHeader;
/// use rtsp::header::types::accept_ranges::RangeFormat;
/// use rtsp::header::types::Accept;
/// use rtsp::header::types::accept::*;
/// use rtsp::header::value::HeaderValue;
/// use mime::*;
/// use std::str::FromStr;
/// let raw_header: Vec<HeaderValue> = vec![];
/// assert_eq!(Accept::decode(&mut raw_header.iter()).unwrap(), None);
///
/// let typed_header = vec![MediaType::new(mime::STAR_STAR, None), MediaType::new(Mime::from_str("video/mp4").unwrap(), Some(QualityParam::new(0.5)))]
///     .into_iter()
///     .collect::<Accept>();
/// let raw_header = vec![HeaderValue::try_from("*/*, video/mp4;q=0.5").unwrap()];
/// assert_eq!(
///     Accept::decode(&mut raw_header.iter()).unwrap(),
///     Some(typed_header)
/// ); 
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Accept(LinkedHashSet<MediaType>);/// use rtsp::header::types::accept::*;


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AcceptError(pub &'static str);

impl Deref for Accept {
    type Target = LinkedHashSet<MediaType>;

    fn deref(&self) -> &LinkedHashSet<MediaType> {
        &self.0
    }
}

impl DerefMut for Accept {
    fn deref_mut(&mut self) -> &mut LinkedHashSet<MediaType> {
        &mut self.0
    }
}

impl FromIterator<MediaType> for Accept {
    fn from_iter<TIterator>(iterator: TIterator) -> Self
    where
        TIterator: IntoIterator<Item = MediaType>,
    {
        Accept(LinkedHashSet::from_iter(iterator))
    }
}

impl TypedHeader for Accept {
    type DecodeError = AcceptError;

    fn decode<'header, Iter>(values: &mut Iter) -> Result<Option<Self>, Self::DecodeError>
    where
        Iter: Iterator<Item = &'header HeaderValue>,
    {
        let mut accept = LinkedHashSet::new();
        let mut present = false;
        for value in values {
            let media = value.as_str().split(',');
            for mtype in media {
                let t = MediaType::try_from(syntax::trim_whitespace(mtype))?;
                accept.insert(t);
            }
            present = true;
        }
        if present {
            Ok(Some(Accept(accept)))
        } else {
            Ok(None)
        }
    }
    
    /// Converts the [`Accept`] type to raw header values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::header::map::TypedHeader;
    /// use rtsp::header::types::accept::MediaType;
    /// use rtsp::header::types::Accept;
    /// use rtsp::header::value::HeaderValue;
    /// use rtsp::header::types::accept::QualityParam;
    /// use mime::*;
    /// use std::str::FromStr;
    /// 
    /// let typed_header = vec![MediaType::new(mime::STAR_STAR, None), MediaType::new(Mime::from_str("video/*").unwrap(), Some(QualityParam::new(0.5)))]
    ///     .into_iter()
    ///     .collect::<Accept>();
    /// let expected_raw_headers = vec![
    ///     vec![HeaderValue::try_from("*/*, video/* ;q=0.5").unwrap()],
    ///     vec![HeaderValue::try_from("video/* ;q=0.5, */*").unwrap()],
    /// ];
    /// let mut raw_header = vec![];
    /// typed_header.encode(&mut raw_header);
    /// assert!(raw_header == expected_raw_headers[0] ||
    ///         raw_header == expected_raw_headers[1]);
    /// ```
    fn encode<Target>(&self, values: &mut Target)
    where
        Target: Extend<HeaderValue>
    {
        // Unsafe Justification
        //
        // Header values must be valid UTF-8, and since we know that the [`RangeFormat`] type
        // guarantees valid ASCII-US (with no newlines), it satisfies the constraints.
        let value = self.iter().map(MediaType::as_str).join(", ");
        values.extend(once(unsafe { HeaderValue::from_string_unchecked(value) }));    }

    fn header_name() -> &'static HeaderName
    where   
        Self: Sized
    {
        &HeaderName::Accept
    }
}

/*
    Each media type or range MAY be followed by one or more accept-
    params, beginning with the "q" parameter to indicate a relative
    quality factor.  The first "q" parameter (if any) separates the media
    type or range's parameters from the accept-params.  Quality factors
    allow the user or user agent to indicate the relative degree of
    preference for that media type, using the qvalue scale from 0 to 1
    (Section 5.3.1 of [RFC7231]).  The default value is q=1.
*/
#[derive(Clone, Debug, PartialEq)]
pub struct MediaType{
    m_type: Mime,
    quality: Option<QualityParam>

} //mtype and msubtype followed by what may be a quality. Then the quality may be followed by a generic param....

impl Default for MediaType {

    fn default() -> Self {
        MediaType{
            m_type: mime::TEXT_PLAIN,
            quality: None
        }
    }
}

impl Eq for MediaType {}

impl Hash for MediaType {

    fn hash<H: Hasher>(&self, state: &mut H) {
        self.m_type.hash(state);
    }
}

impl MediaType{

    pub fn new(m_type: Mime, quality: Option<QualityParam>) -> Self {
        MediaType{
            m_type,
            quality
        }
    }

    /// Returns a `&str` representation of the media type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::header::types::accept::MediaType;
    /// use rtsp::header::types::accept::QualityParam;
    /// use mime::*;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(MediaType::new(mime::STAR_STAR, None).as_str(), "*/*");
    /// assert_eq!(MediaType::new(Mime::from_str("video/*").unwrap(), Some(QualityParam::new(0.5))).as_str(), "video/* ;q=0.5");
    /// ```
    pub fn as_str(&self) -> String {
        let m_type = self.m_type.to_string();
        match &self.quality {
            Some(q) => format!("{} ;{}", m_type, q.as_str()),
            None => format!("{}", m_type)
        }
    }
}

impl<'accept> TryFrom<&'accept [u8]> for MediaType {
    type Error = AcceptError;

    fn try_from(value: &'accept [u8]) -> Result<Self, Self::Error> {
        let mut split = value.split(|element| element.clone() == ';' as u8);
        let mut decoded = MediaType::default();
        if let Some(mediatype) = split.next() {
            let mtype: Mime = str::from_utf8(mediatype).unwrap().parse().unwrap();
            decoded.m_type = mtype;
            if let Some(quality) = split.next() {
                let q = QualityParam::try_from(quality).unwrap();
                decoded.quality = Some(q);
            }
        }
        Ok(decoded)
    }
}

impl<'accept> TryFrom<&'accept str> for MediaType {
    type Error = AcceptError;

    fn try_from(value: &'accept str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        Self::try_from(bytes)
    } 
}

#[derive(Clone, Debug, PartialEq)]
pub struct QualityParam(f32);

impl<'quality> TryFrom<&'quality [u8]> for QualityParam {
    type Error = AcceptError;

    fn try_from(value: &'quality [u8]) -> Result<Self, Self::Error> {
        let stringify = String::from_utf8(value.to_vec()).unwrap();
        let val = stringify.split("=").collect::<Vec<&str>>();
        if val.len() < 2 || val.len() > 2 {
            return Err(AcceptError("incorrect quality format"))
        }        
        let quality = val[1].parse::<f32>();
        match quality {
            Ok(q) =>  return Ok(QualityParam::new(q)),
            Err(_) => return Err(AcceptError("unable to parse f32 from string"))
        }
    }
}

impl<'quality> TryFrom<&'quality str> for QualityParam {
    type Error = AcceptError;

    fn try_from(value: &'quality str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        Self::try_from(bytes)
    }
}

impl QualityParam {

    pub fn new(q_value: f32) -> Self {
        let q_value = q_value.clamp(0.0_f32, 1.0_f32);
        QualityParam(q_value)
    }


    /// Returns a `String` representation of the quality parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    ///
    /// use rtsp::header::types::accept::QualityParam;
    ///
    /// assert_eq!(QualityParam::new(0.5).as_str(), "q=0.5");
    /// ```
    pub fn as_str(&self) -> String {
        let q_val = self.0.to_string();
        format!("q={}", q_val)
    }
}

pub struct MediaTypeParseError(String);
