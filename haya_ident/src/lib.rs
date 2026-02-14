#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use core::str::from_utf8_unchecked;
use haya_str::HayaStr;
use mser::{ByteArray, Error, Read, UnsafeWriter, V21, Write};

pub const MINECRAFT: &str = "minecraft";

/// Determines whether a byte is allowed in an identifier path segment.

///

/// # Returns

///

/// `true` if the byte is one of ASCII lowercase letters `a`–`z`, digits `0`–`9`, `_`, `-`, `.`, or `/`; `false` otherwise.

///

/// # Examples

///

/// ```

/// assert!(is_valid_path(b'a'));

/// assert!(is_valid_path(b'0'));

/// assert!(is_valid_path(b'_'));

/// assert!(!is_valid_path(b'A'));

/// assert!(!is_valid_path(b' '));

/// ```
const fn is_valid_path(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/')
}

/// Determines whether a byte is allowed in a namespace (lowercase ASCII letters, digits, `_`, `-`, or `.`).
///
/// Returns `true` if the byte is one of `a`..`z`, `0`..`9`, `_`, `-`, or `.`, `false` otherwise.
///
/// # Examples
///
/// ```
/// assert!(is_valid_namespace(b'a'));
/// assert!(is_valid_namespace(b'0'));
/// assert!(is_valid_namespace(b'_'));
/// assert!(!is_valid_namespace(b'A'));
/// ```
const fn is_valid_namespace(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.')
}

/// Splits a byte slice at the first colon (`:`).
///
/// # Examples
///
/// ```
/// let (a, b) = split_once(b"ns:path").unwrap();
/// assert_eq!(a, b"ns");
/// assert_eq!(b, b"path");
/// assert!(split_once(b"noprefix").is_none());
/// ```
///
/// # Returns
///
/// `Some((before, after))` where `before` is the bytes before the first `:` and `after` is the bytes after it; `None` if the slice contains no `:`.
fn split_once(n: &[u8]) -> Option<(&[u8], &[u8])> {
    let index = n.iter().position(|&x| x == b':')?;
    Some((&n[..index], &n[index + 1..]))
}

/// Parse a byte slice as a Minecraft-style identifier, producing an optional namespace and a path.
///
/// The input must be ASCII and conform to identifier validation rules; otherwise the function returns `None`.
///
/// # Parameters
///
/// - `ident`: Byte slice containing the identifier to parse (e.g. `b"minecraft:stone"` or `b"mod:stone"`).
///
/// # Returns
///
/// `Some((namespace, path))` when the input is a valid identifier:
/// - `namespace` is `Some(ns)` if an explicit namespace was provided, or `None` when the default (`"minecraft"`) applies.
/// - `path` is the identifier path string.
/// Returns `None` if the input contains non-ASCII bytes or fails validation.
///
/// # Examples
///
/// ```
/// assert_eq!(parse_ident(b"minecraft:stone"), Some((None, "stone")));
/// assert_eq!(parse_ident(b"mod:stone"), Some((Some("mod"), "stone")));
/// assert_eq!(parse_ident(b"invalid:UPPER"), None);
/// ```
pub fn parse_ident(ident: &[u8]) -> Option<(Option<&str>, &str)> {
    if !ident.is_ascii() {
        return None;
    }
    parse_ident_ascii(ident)
}

/// Parse an ASCII identifier in Minecraft `namespace:path` form.
///
/// Attempts to interpret `ident` as an ASCII identifier. Handles three forms:
/// - `minecraft:<path>` — treated as no explicit namespace, returns `None` for the namespace.
/// - `<namespace>:<path>` — returns `Some(namespace)` if the namespace is non-empty.
/// - `<path>` (no colon) — treated as no explicit namespace.
///
/// # Parameters
///
/// - `ident`: ASCII byte slice containing the identifier to parse.
///
/// # Returns
///
/// `Some((namespace, path))` on successful validation where `namespace` is `Some(&str)` when an explicit, non-empty namespace was present and `None` when no namespace was specified or the `minecraft:` prefix was used; `path` is the validated path string. Returns `None` if validation fails.
///
/// # Examples
///
/// ```
/// assert_eq!(parse_ident_ascii(b"minecraft:stone"), Some((None, "stone")));
/// assert_eq!(parse_ident_ascii(b"mod:items/sword"), Some((Some("mod"), "items/sword")));
/// assert_eq!(parse_ident_ascii(b"items/sword"), Some((None, "items/sword")));
/// ```
fn parse_ident_ascii(ident: &[u8]) -> Option<(Option<&str>, &str)> {
    match ident.strip_prefix(b"minecraft:") {
        Some(path) => unsafe {
            if path.iter().copied().all(is_valid_path) {
                Some((None, from_utf8_unchecked(path)))
            } else {
                None
            }
        },
        None => match split_once(ident) {
            Some((ns, path)) => unsafe {
                if ns.iter().copied().all(is_valid_namespace)
                    && path.iter().copied().all(is_valid_path)
                {
                    Some((
                        if !ns.is_empty() {
                            Some(from_utf8_unchecked(ns))
                        } else {
                            None
                        },
                        from_utf8_unchecked(path),
                    ))
                } else {
                    None
                }
            },
            None => unsafe {
                if ident.iter().copied().all(is_valid_path) {
                    Some((None, from_utf8_unchecked(ident)))
                } else {
                    None
                }
            },
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ident<'a> {
    namespace: Option<&'a str>,
    path: &'a str,
}

impl<'a> Ident<'a> {
    /// The namespace component of the identifier, if present.
    
    ///
    
    /// # Examples
    
    ///
    
    /// ```
    
    /// let id = Ident { namespace: Some("foo"), path: "bar" };
    
    /// assert_eq!(id.namespace(), Some("foo"));
    
    /// ```
    pub fn namespace(&self) -> Option<&str> {
        self.namespace
    }

    /// Accesses the identifier's path component.
    ///
    /// # Examples
    ///
    /// ```
    /// let ident = Ident { namespace: Some("example"), path: "stone" };
    /// let id = Identifier::new(ident);
    /// assert_eq!(id.path(), "stone");
    /// ```
    pub fn path(&self) -> &str {
        self.path
    }
}

impl<'a> Read<'a> for Ident<'a> {
    /// Deserialize an `Ident` from the buffer, consuming its length-prefixed identifier bytes.
    ///
    /// Reads a length-prefixed identifier from `buf`, validates its namespace and path, and returns
    /// a parsed `Ident` on success. Advances `buf` past the bytes consumed.
    ///
    /// # Parameters
    ///
    /// - `buf`: remaining input buffer; advanced past the identifier when read succeeds.
    ///
    /// # Returns
    ///
    /// `Ok(Ident)` with the parsed namespace and path on success, `Err(Error)` if reading fails or the identifier is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a buffer containing a V21-length-prefixed identifier "minecraft:stone"
    /// use crate::{Ident, MINECRAFT};
    /// // `UnsafeWriter`/V21 encoding is not shown here; this example illustrates usage only.
    /// let bytes: &[u8] = &[]; // replace with a real encoded identifier for a real test
    /// let mut buf = bytes;
    /// // let id = Ident::read(&mut buf).unwrap();
    /// // assert_eq!(id.path(), "stone");
    /// ```
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let identifier = ByteArray::<32767>::read(buf)?.0;
        match parse_ident(identifier) {
            Some((namespace, path)) => Ok(Self { namespace, path }),
            None => Err(Error),
        }
    }
}

impl Write for Ident<'_> {
    /// Writes the identifier to `w` as a V21-length-prefixed ASCII string in the form `namespace:path`, using `"minecraft"` as the namespace when none is set.
    ///
    /// # Safety
    ///
    /// The function is `unsafe` because it forwards raw writes to an `UnsafeWriter`. Callers must ensure `w` is a valid, properly-initialized `UnsafeWriter` and that performing the write does not violate any of `UnsafeWriter`'s safety invariants (for example, no concurrent mutable access to the same output buffer).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Writes "minecraft:stone" (with length prefix) when namespace is None
    /// let ident = Ident { namespace: None, path: "stone" };
    /// let mut out = UnsafeWriter::new(...);
    /// unsafe { ident.write(&mut out); }
    /// ```
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let namespace = match self.namespace {
                Some(x) => x,
                None => MINECRAFT,
            };
            V21((namespace.len() + 1 + self.path.len()) as _).write(w);
            w.write(namespace.as_bytes());
            w.write_byte(b':');
            w.write(self.path.as_bytes());
        }
    }

    /// Compute the total number of bytes required to serialize this identifier.
    ///
    /// The result equals the length in bytes of the V21-encoded payload length for the string
    /// "namespace:path" (where `namespace` defaults to "minecraft" when `self.namespace` is `None`)
    /// plus the payload byte length itself.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct an identifier and measure its serialized size.
    /// let ident = Ident { namespace: Some("example"), path: "stone" };
    /// let size = ident.len_s();
    /// assert!(size >= "example:stone".len() + 1); // at least the payload plus a small prefix
    /// ```
    fn len_s(&self) -> usize {
        let namespace = match self.namespace {
            Some(x) => x,
            None => MINECRAFT,
        };
        let a = namespace.len() + 1 + self.path.len();
        V21(a as u32).len_s() + a
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier(Inner);

impl Identifier {
    /// Constructs an owned `Identifier` from an `Ident`, choosing an internal representation
    /// that stores the namespace and path efficiently.
    ///
    /// The returned `Identifier` takes ownership of the identifier data: if the input `Ident`
    /// contains a namespace, both namespace and path are stored as owned boxed strings;
    /// otherwise the path is stored in a compact or heap-backed form depending on whether
    /// it can be represented by `HayaStr`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Full (namespace + path)
    /// let id = Ident { namespace: Some("my_ns"), path: "my/path" };
    /// let owned = Identifier::new(id);
    /// assert_eq!(owned.namespace(), Some("my_ns"));
    /// assert_eq!(owned.path(), "my/path");
    ///
    /// // No namespace -> path-only representation
    /// let id2 = Ident { namespace: None, path: "only_path" };
    /// let owned2 = Identifier::new(id2);
    /// assert_eq!(owned2.namespace(), None);
    /// assert_eq!(owned2.path(), "only_path");
    /// ```
    pub fn new(ident: Ident) -> Self {
        let Ident { namespace, path } = ident;
        match namespace {
            Some(namespace) => {
                let namespace = namespace.to_owned().into_boxed_str();
                let path = path.to_owned().into_boxed_str();
                Self(Inner::Full { namespace, path })
            }
            None => match HayaStr::new(path) {
                Ok(path) => Self(Inner::Thin { path }),
                Err(_) => Self(Inner::Heap {
                    path: path.to_owned().into_boxed_str(),
                }),
            },
        }
    }

    /// Returns the path component of the identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// let ident = Ident { namespace: None, path: "stone" };
    /// let id = Identifier::new(ident);
    /// assert_eq!(id.path(), "stone");
    /// ```
    pub fn path(&self) -> &str {
        match &self.0 {
            Inner::Thin { path } => path,
            Inner::Heap { path } => path,
            Inner::Full { path, .. } => path,
        }
    }

    /// Returns the stored namespace for this identifier when present.
    ///
    /// The identifier may store a namespace only in the `Full` representation; this returns that namespace if available.
    ///
    /// # Examples
    ///
    /// ```
    /// let ident = Ident { namespace: Some("minecraft"), path: "stone" };
    /// let id = Identifier::new(ident);
    /// assert_eq!(id.namespace(), Some("minecraft"));
    /// ```
    pub fn namespace(&self) -> Option<&str> {
        match &self.0 {
            Inner::Thin { .. } => None,
            Inner::Heap { .. } => None,
            Inner::Full { namespace, .. } => Some(namespace),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Inner {
    Thin { path: HayaStr },
    Heap { path: Box<str> },
    Full { namespace: Box<str>, path: Box<str> },
}