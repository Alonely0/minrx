#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
use std::{
    error::Error,
    ffi::c_int,
    fmt::Display,
    mem::MaybeUninit,
    ops::{BitAnd, BitOr, Not},
    ptr::NonNull,
    range::Range,
};

use minrx_sys::{
    minrx_regcomp_flags_t, minrx_regcomp_flags_t_MINRX_REG_BRACE_COMPAT,
    minrx_regcomp_flags_t_MINRX_REG_BRACK_ESCAPE, minrx_regcomp_flags_t_MINRX_REG_EXTENDED,
    minrx_regcomp_flags_t_MINRX_REG_EXTENSIONS_BSD, minrx_regcomp_flags_t_MINRX_REG_EXTENSIONS_GNU,
    minrx_regcomp_flags_t_MINRX_REG_ICASE, minrx_regcomp_flags_t_MINRX_REG_MINDISABLE,
    minrx_regcomp_flags_t_MINRX_REG_MINIMAL, minrx_regcomp_flags_t_MINRX_REG_NATIVE1B,
    minrx_regcomp_flags_t_MINRX_REG_NEWLINE, minrx_regcomp_flags_t_MINRX_REG_NOSUB, minrx_regerror,
    minrx_regex_t, minrx_regexec_flags_t, minrx_regexec_flags_t_MINRX_REG_FIRSTSUB,
    minrx_regexec_flags_t_MINRX_REG_NOFIRSTBYTES, minrx_regexec_flags_t_MINRX_REG_NOSUBRESET,
    minrx_regexec_flags_t_MINRX_REG_NOTBOL, minrx_regexec_flags_t_MINRX_REG_NOTEOL,
    minrx_regexec_flags_t_MINRX_REG_RESUME, minrx_regfree, minrx_regmatch_t, minrx_regncomp,
    minrx_regnexec, minrx_result_t_MINRX_REG_BADBR, minrx_result_t_MINRX_REG_BADPAT,
    minrx_result_t_MINRX_REG_BADRPT, minrx_result_t_MINRX_REG_EBRACE,
    minrx_result_t_MINRX_REG_EBRACK, minrx_result_t_MINRX_REG_ECOLLATE,
    minrx_result_t_MINRX_REG_ECTYPE, minrx_result_t_MINRX_REG_EESCAPE,
    minrx_result_t_MINRX_REG_EPAREN, minrx_result_t_MINRX_REG_ERANGE,
    minrx_result_t_MINRX_REG_ESPACE, minrx_result_t_MINRX_REG_ESUBREG,
    minrx_result_t_MINRX_REG_NOMATCH, minrx_result_t_MINRX_REG_SUCCESS,
    minrx_result_t_MINRX_REG_UNKNOWN,
};

/// A MinRX regex matcher. [`Send`] but not [`Sync`].
#[repr(transparent)]
#[must_use = "This value does nothing on its own -- you must call its methods
              to start matching."]
pub struct Regex(minrx_regex_t);

/// A match on some haystack. It is a wrapper of [`std::range::Range<usize>`]
/// with some convenience methods and derives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Match {
    pub start: usize,
    pub end: usize,
}

/// An iterator over all matches on a haystack.
#[must_use = "This value does nothing on its own -- you must advance the \
              iterator to start matching."]
pub struct MatchIter<'r, 'h> {
    regex: &'r Regex,
    haystack: &'h [u8],
    rm: [minrx_regmatch_t; 1],
    options: MatchOptions,
    resuming: bool,
    is_done: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[must_use = "This value does nothing on its own -- you must consume it with \
              `Self::build()`. It is also `Copy` and its setters return \
              `Self`, so you must assign it or consume their values."]
pub struct RegexBuilder(minrx_regcomp_flags_t);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[must_use = "This value does nothing on its own -- you must consume it with \
              `Regex::*_with()`. It is also `Copy` and its setters return \
              `Self`, so you must assign it or consume their values."]
pub struct MatchOptions(minrx_regexec_flags_t);

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuildError {
    BadPattern(String) = minrx_result_t_MINRX_REG_BADPAT,
    BadBracket(String) = minrx_result_t_MINRX_REG_BADBR,
    BadRepetition(String) = minrx_result_t_MINRX_REG_BADRPT,
    UnbalancedBrace(String) = minrx_result_t_MINRX_REG_EBRACE,
    UnbalancedBracket(String) = minrx_result_t_MINRX_REG_EBRACK,
    InvalidCollate(String) = minrx_result_t_MINRX_REG_ECOLLATE,
    InvalidClass(String) = minrx_result_t_MINRX_REG_ECTYPE,
    InvalidEscape(String) = minrx_result_t_MINRX_REG_EESCAPE,
    UnbalancedParen(String) = minrx_result_t_MINRX_REG_EPAREN,
    InvalidEndpoint(String) = minrx_result_t_MINRX_REG_ERANGE,
    AllocError(String) = minrx_result_t_MINRX_REG_ESPACE,
    InvalidDigitEscape(String) = minrx_result_t_MINRX_REG_ESUBREG,
    Unknown(String) = minrx_result_t_MINRX_REG_UNKNOWN,
}

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MatchError {
    AllocError(String) = minrx_result_t_MINRX_REG_ESPACE,
    Unknown(String) = minrx_result_t_MINRX_REG_UNKNOWN,
}

impl Regex {
    /// Constructs a new [`Regex`] with the default options. Look at
    /// [`RegexBuilder`] for information about configuration options.
    ///
    /// # Errors
    ///
    /// This method returns an error if the regular expression can't be properly
    /// parsed or if there is an allocation failure.
    pub fn new(pattern: impl AsRef<[u8]>) -> Result<Self, BuildError> {
        RegexBuilder::new().build(pattern)
    }

    /// Returns the number of captures the pattern generated.
    #[must_use]
    pub fn capture_count(&self) -> usize {
        self.0.re_nsub
    }

    /// Returns matches for all captures, if any. Its behavior can be customized
    /// with [`Self::find_matches_with`].
    ///
    /// # Errors
    ///
    /// This method returns an error if there is an internal failure of the
    /// MinRX library, or an allocation failure (it ought not to allocate at
    /// this stage, though). Therefore, this is unlikely to error.
    pub fn find_matches(
        &self,
        haystack: impl AsRef<[u8]>,
    ) -> Result<Option<Box<[Option<Match>]>>, MatchError> {
        self.find_matches_with(haystack, MatchOptions::new())
    }

    /// Returns if the pattern matches any substring. It is recommended you
    /// toggle [`RegexBuilder::no_substrings`] for better performance, if all
    /// you need is to check existance. Its behavior can be customized with
    /// [`Self::is_match_with`].
    ///
    /// # Errors
    ///
    /// This method returns an error if there is an internal failure of the
    /// MinRX library, or an allocation failure (it ought not to allocate at
    /// this stage, though). Therefore, this is unlikely to error.
    pub fn is_match(&self, haystack: impl AsRef<[u8]>) -> Result<bool, MatchError> {
        self.is_match_with(haystack, MatchOptions::new())
    }

    /// Returns matches for all captures, if any. Allows for some execution
    /// options.
    ///
    /// # Errors
    ///
    /// This method returns an error if there is an internal failure of the
    /// MinRX library, or an allocation failure (it ought not to allocate at
    /// this stage, though). Therefore, this is unlikely to error.
    pub fn find_matches_with(
        &self,
        haystack: impl AsRef<[u8]>,
        options: MatchOptions,
    ) -> Result<Option<Box<[Option<Match>]>>, MatchError> {
        let haystack = haystack.as_ref();
        let mut buf = Vec::with_capacity(self.0.re_nsub + 1);
        let slice =
            NonNull::slice_from_raw_parts(NonNull::from(buf.as_slice()).cast(), buf.capacity());
        let res = unsafe { self.regnexec(haystack, Some(slice), options) };

        res.map(|found| {
            found.then(|| {
                unsafe { buf.set_len(buf.capacity()) };
                buf.into_iter()
                    .map(|m: minrx_regmatch_t| {
                        Some(Match {
                            start: m.rm_so.try_into().ok()?,
                            end: m.rm_eo.try_into().ok()?,
                        })
                    })
                    .collect()
            })
        })
    }

    /// Returns if the pattern matches any substring. It is recommended you
    /// toggle [`RegexBuilder::no_substrings`] for better performance, if all
    /// you need is to check existance. Allows for some execution options.
    ///
    /// # Errors
    ///
    /// This method returns an error if there is an internal failure of the
    /// MinRX library, or an allocation failure (it ought not to allocate at
    /// this stage, though). Therefore, this is unlikely to error.
    pub fn is_match_with(
        &self,
        haystack: impl AsRef<[u8]>,
        options: MatchOptions,
    ) -> Result<bool, MatchError> {
        unsafe { self.regnexec(haystack, None, options) }
    }

    /// Returns an iterator over all matches of the pattern. Its behavior can
    /// be customized with [`Self::find_matches_with`].
    pub fn find_iter<'r, 'h>(
        &'r self,
        haystack: &'h (impl AsRef<[u8]> + ?Sized),
    ) -> MatchIter<'r, 'h> {
        self.find_iter_with_flags(haystack, MatchOptions::new())
    }

    /// Returns an iterator over all matches of the pattern. Allows for some
    /// execution options.
    pub fn find_iter_with_flags<'r, 'h>(
        &'r self,
        haystack: &'h (impl AsRef<[u8]> + ?Sized),
        options: MatchOptions,
    ) -> MatchIter<'r, 'h> {
        MatchIter {
            regex: self,
            haystack: haystack.as_ref(),
            rm: [minrx_regmatch_t { rm_so: 0, rm_eo: 0 }],
            options,
            resuming: false,
            is_done: false,
        }
    }

    /// Internal utility.
    unsafe fn regnexec(
        &self,
        haystack: impl AsRef<[u8]>,
        buf: Option<NonNull<[minrx_regmatch_t]>>,
        options: MatchOptions,
    ) -> Result<bool, MatchError> {
        let haystack = haystack.as_ref();
        let (buf_ptr, buf_cap) = buf.map_or_default(|b| (b.as_ptr().cast(), b.len()));

        let res = unsafe {
            minrx_regnexec(
                (&raw const self.0).cast_mut(),
                haystack.len(),
                haystack.as_ptr(),
                buf_cap,
                buf_ptr,
                options.as_c_int(),
            )
        };

        MatchError::from_raw(res, self)
    }
}

impl RegexBuilder {
    /// Creates a new [`RegexBuilder`] that can be freely reused. This struct
    /// configures [`Regex`], and constructs it with [`RegexBuilder::build`].
    pub fn new() -> Self {
        Self(minrx_regcomp_flags_t_MINRX_REG_EXTENDED)
    }

    /// Attempts to build a new [`Regex`] with the given pattern and options.
    ///
    /// # Errors
    ///
    /// This method returns an error if the regular expression can't be properly
    /// parsed or there is an allocation failure.
    pub fn build(self, pattern: impl AsRef<[u8]>) -> Result<Regex, BuildError> {
        let pattern = pattern.as_ref();
        let mut regex = MaybeUninit::zeroed();
        let res = unsafe {
            minrx_regncomp(
                regex.as_mut_ptr(),
                pattern.len(),
                pattern.as_ptr(),
                self.as_c_int(),
            )
        };
        BuildError::from_raw(res, &mut regex)?;

        let regex = unsafe { regex.assume_init() };
        Ok(Regex(regex))
    }

    /// Uses extended POSIX syntax. This is enabled by default, and in the
    /// current version of MinRX, disabling it is a no-op. In the meantime,
    /// this function is a placeholder.
    pub fn extended(self, _enable: bool) -> Self {
        self
    }

    /// Ignore case in both pattern and search.
    pub fn case_insensitive(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regcomp_flags_t_MINRX_REG_ICASE, enable);
        self
    }

    /// Swap meaning of operators `?` and `??`, `*` and `*?`, and `+` and `+?`.
    pub fn swap_greed(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regcomp_flags_t_MINRX_REG_MINIMAL, enable);
        self
    }

    /// Excludes `\n` from `.` and `[^...]`; treat as boundary for `^` and `$`.
    pub fn multi_line(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regcomp_flags_t_MINRX_REG_NEWLINE, enable);
        self
    }

    /// Output true/false results only; no [`Match`] substring results.
    pub fn no_substrings(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regcomp_flags_t_MINRX_REG_NOSUB, enable);
        self
    }

    /// `{` begins interval expression only when followed by digit.
    pub fn brace_compat(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regcomp_flags_t_MINRX_REG_BRACE_COMPAT, enable);
        self
    }

    /// Bracket expressions `[...]` allow backslash escapes.
    pub fn escapes_in_brackets(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regcomp_flags_t_MINRX_REG_BRACK_ESCAPE, enable);
        self
    }

    /// Enable BSD extensions: `\<` and `\>`.
    pub fn bsd_extensions(mut self, enable: bool) -> Self {
        self.0 = mask(
            self.0,
            minrx_regcomp_flags_t_MINRX_REG_EXTENSIONS_BSD,
            enable,
        );
        self
    }

    /// Enable GNU extensions: `\b`, `\B`, `\s`, `\S`, `\w`, `\W`.
    pub fn gnu_extensions(mut self, enable: bool) -> Self {
        self.0 = mask(
            self.0,
            minrx_regcomp_flags_t_MINRX_REG_EXTENSIONS_GNU,
            enable,
        );
        self
    }

    /// Use native encoding for 8-bit character sets.
    pub fn native_encoding(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regcomp_flags_t_MINRX_REG_NATIVE1B, enable);
        self
    }

    /// Disable POSIX 2024 minimal repetitions.
    pub fn disable_min_reps(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regcomp_flags_t_MINRX_REG_MINDISABLE, enable);
        self
    }

    /// Internal utility.
    fn as_c_int(self) -> c_int {
        self.0.cast_signed()
    }
}

impl MatchOptions {
    /// Internal utility.
    fn as_c_int(self) -> c_int {
        self.0.cast_signed()
    }

    /// Creates a new [`MatchOptions`] that can be freely reused. This struct
    /// configures [`Regex`] matching, and can be used with
    /// [`Regex::find_matches_with`] and [`Regex::is_match_with`]. Their
    /// non-`_with` counterparts use the default value of this.
    pub fn new() -> Self {
        Self(0)
    }

    /// Disables matching `^` at the beginning of the string.
    pub fn not_bol(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regexec_flags_t_MINRX_REG_NOTBOL, enable);
        self
    }

    /// Disables matching `$` at the end of the string.
    pub fn not_eol(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regexec_flags_t_MINRX_REG_NOTEOL, enable);
        self
    }

    /// repeated subexpressions capture their first occurrence (rather than last).
    pub fn first_subexpr(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regexec_flags_t_MINRX_REG_FIRSTSUB, enable);
        self
    }

    /// Repeated subexpressions don't clear their contained subexpressions.
    pub fn no_subexpr_reset(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regexec_flags_t_MINRX_REG_NOSUBRESET, enable);
        self
    }

    /// Resumes matching at the end of the last match. Private on purpose;
    /// cannot be safely used without extra care.
    fn resume(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regexec_flags_t_MINRX_REG_RESUME, enable);
        self
    }

    /// Disables rapid skip-ahead over impossible first bytes.
    pub fn no_first_bytes(mut self, enable: bool) -> Self {
        self.0 = mask(self.0, minrx_regexec_flags_t_MINRX_REG_NOFIRSTBYTES, enable);
        self
    }
}

impl Default for RegexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MatchOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildError {
    fn from_raw(res: c_int, regex: &mut MaybeUninit<minrx_regex_t>) -> Result<(), Self> {
        let err = || regerror(res, regex.as_ptr());
        let err = match res.cast_unsigned() {
            res if res == minrx_result_t_MINRX_REG_SUCCESS => return Ok(()),
            res if res == minrx_result_t_MINRX_REG_BADPAT => Err(Self::BadPattern(err())),
            res if res == minrx_result_t_MINRX_REG_BADBR => Err(Self::BadBracket(err())),
            res if res == minrx_result_t_MINRX_REG_BADRPT => Err(Self::BadRepetition(err())),
            res if res == minrx_result_t_MINRX_REG_EBRACE => Err(Self::UnbalancedBrace(err())),
            res if res == minrx_result_t_MINRX_REG_EBRACK => Err(Self::UnbalancedBracket(err())),
            res if res == minrx_result_t_MINRX_REG_ECOLLATE => Err(Self::InvalidCollate(err())),
            res if res == minrx_result_t_MINRX_REG_ECTYPE => Err(Self::InvalidClass(err())),
            res if res == minrx_result_t_MINRX_REG_EESCAPE => Err(Self::InvalidEscape(err())),
            res if res == minrx_result_t_MINRX_REG_EPAREN => Err(Self::UnbalancedParen(err())),
            res if res == minrx_result_t_MINRX_REG_ERANGE => Err(Self::InvalidEndpoint(err())),
            res if res == minrx_result_t_MINRX_REG_ESPACE => return Err(Self::AllocError(err())),
            res if res == minrx_result_t_MINRX_REG_ESUBREG => Err(Self::InvalidDigitEscape(err())),
            _ => Err(Self::Unknown(err())),
        };
        drop(Regex(unsafe { regex.assume_init() }));
        err
    }
}

impl MatchError {
    fn from_raw(res: c_int, regex: &Regex) -> Result<bool, Self> {
        let err = || regerror(res, &raw const regex.0);
        match res.cast_unsigned() {
            res if res == minrx_result_t_MINRX_REG_SUCCESS => Ok(true),
            res if res == minrx_result_t_MINRX_REG_NOMATCH => Ok(false),
            res if res == minrx_result_t_MINRX_REG_ESPACE => Err(Self::AllocError(err())),
            _ => Err(Self::Unknown(err())),
        }
    }
}

impl Iterator for MatchIter<'_, '_> {
    type Item = Result<Match, MatchError>;

    /// Advances to the next match.
    ///
    /// # Errors
    ///
    /// This method returns an error if there is an internal failure of the
    /// MinRX library, or an allocation failure (it ought not to allocate at
    /// this stage, though). Therefore, this is unlikely to error.
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        self.options = self.options.resume(self.resuming);

        let res = unsafe {
            self.regex.regnexec(
                self.haystack,
                Some(NonNull::from(&mut self.rm)),
                self.options,
            )
        };

        match res {
            Ok(true) => {
                let so = self.rm[0].rm_so.cast_unsigned();
                let eo = self.rm[0].rm_eo.cast_unsigned();

                if so == eo {
                    if eo >= self.haystack.len() {
                        self.is_done = true;
                    } else {
                        // MINRX_REG_RESUME only repositions when `rm_eo > 0`,
                        // so an empty match would otherwise be found again
                        // forever. This bumps it forward one character.
                        self.rm[0].rm_eo += 1;
                    }
                }

                self.resuming = true;
                Some(Ok(Match { start: so, end: eo }))
            }
            Ok(false) => {
                self.is_done = true;
                None
            }
            Err(e) => {
                self.is_done = true;
                Some(Err(e))
            }
        }
    }
}

#[inline]
fn mask<T>(set: T, bit: T, enable: bool) -> T
where
    T: BitOr<Output = T> + BitAnd<Output = T> + Not<Output = T>,
{
    if enable { set | bit } else { set & !bit }
}

fn regerror(res: c_int, regex: *const minrx_regex_t) -> String {
    let mut buf = Vec::with_capacity(53);
    let new_len = unsafe { minrx_regerror(res, regex, buf.as_mut_ptr(), buf.capacity()) };

    if new_len > buf.capacity() {
        buf.reserve_exact(new_len);
        unsafe { minrx_regerror(res, regex, buf.as_mut_ptr(), buf.capacity()) };
    }

    unsafe { buf.set_len(new_len.saturating_sub(1)) }; // Set len; remove null-terminator.
    String::from_utf8_lossy(&buf).to_string()
}

impl Drop for Regex {
    fn drop(&mut self) {
        unsafe { minrx_regfree(&raw mut self.0) };
    }
}

unsafe impl Send for Regex {}

impl From<Match> for std::ops::Range<usize> {
    fn from(value: Match) -> Self {
        value.start..value.end
    }
}

impl Match {
    /// Returns the components as a [`Range<usize>`] that can be used for slice
    /// indexing. This is equivalent to [`<Self as Into<Range<usize>>::into`],
    /// but takes by-reference; this might be friendlier for usage in Higher-
    /// Kinded Functions (HKFs).
    ///
    /// Note: this is the new Range. You may use its [`From`] impl if you need
    /// the legacy one.
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        (self.start..self.end).into()
    }
}

impl From<Match> for Range<usize> {
    fn from(value: Match) -> Self {
        value.range()
    }
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Regex parse error: ")?;
        match self {
            BuildError::BadPattern(s)
            | BuildError::BadBracket(s)
            | BuildError::BadRepetition(s)
            | BuildError::UnbalancedBrace(s)
            | BuildError::UnbalancedBracket(s)
            | BuildError::InvalidCollate(s)
            | BuildError::InvalidClass(s)
            | BuildError::InvalidEscape(s)
            | BuildError::UnbalancedParen(s)
            | BuildError::InvalidEndpoint(s)
            | BuildError::AllocError(s)
            | BuildError::InvalidDigitEscape(s)
            | BuildError::Unknown(s) => f.write_str(s),
        }
    }
}

impl Display for MatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Regex execution error: ")?;
        match self {
            MatchError::AllocError(s) | MatchError::Unknown(s) => f.write_str(s),
        }
    }
}

impl Error for BuildError {}
impl Error for MatchError {}
