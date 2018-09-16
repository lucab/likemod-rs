use super::errors;
use super::nr;
use errno;
use libc;
use std::{collections, fs};

use std::os::unix::io::AsRawFd;
use std::string::ToString;

/// Parameters used for loading a module, map of name-value.
pub type ModParams = collections::BTreeMap<String, ModParamValue>;

/// A module parameter value.
#[derive(Debug)]
pub enum ModParamValue {
    /// Boolean value.
    Bool(bool),
    /// Integer value, signed 64-bits.
    Int(i64),
    /// String value.
    Str(String),
    /// Array of values.
    Array(Vec<ModParamValue>),
}

impl ToString for ModParamValue {
    fn to_string(&self) -> String {
        match *self {
            ModParamValue::Bool(b) => b.to_string(),
            ModParamValue::Int(n) => n.to_string(),
            ModParamValue::Str(ref s) => s.clone(),
            ModParamValue::Array(ref a) => {
                let mut values = String::new();
                for v in a {
                    if !values.is_empty() {
                        values += ",";
                    }
                    values += &v.to_string();
                }
                values
            }
        }
    }
}

/// Module loader.
#[derive(Debug)]
pub struct ModLoader {
    ignore_modversion: bool,
    ignore_vermagic: bool,
    params: ModParams,
}

impl Default for ModLoader {
    fn default() -> Self {
        Self {
            ignore_modversion: false,
            ignore_vermagic: false,
            params: ModParams::new(),
        }
    }
}

impl ModLoader {
    /// Create a new default `ModLoader`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to avoid checking for a matching modversion.
    pub fn ignore_modversion(mut self, ignored: bool) -> Self {
        self.ignore_modversion = ignored;
        self
    }

    /// Set whether to avoid checking for a matching vermagic.
    pub fn ignore_vermagic(mut self, ignored: bool) -> Self {
        self.ignore_vermagic = ignored;
        self
    }

    /// Set module parameters to be used at load time.
    pub fn set_parameters(mut self, params: ModParams) -> Self {
        self.params = params;
        self
    }

    /// Load a module from a file.
    pub fn load_module_file(self, modfile: &fs::File) -> errors::Result<()> {
        let flags = match (self.ignore_modversion, self.ignore_vermagic) {
            (false, false) => 0,
            (true, false) => nr::IGNORE_MODVERSIONS,
            (false, true) => nr::IGNORE_VERMAGIC,
            (true, true) => nr::IGNORE_MODVERSIONS | nr::IGNORE_VERMAGIC,
        };
        let pp = params_to_string(&self.params);
        let fd = modfile.as_raw_fd();
        // UNSAFE(lucab): required syscall, all parameters are local and immutable.
        let r = unsafe { libc::syscall(nr::FINIT_MODULE, fd, pp.as_ptr(), flags) };
        match r {
            0 => Ok(()),
            _ => Err(
                errors::Error::from_kind(errors::ErrorKind::Sys(errno::errno()))
                    .chain_err(|| "finit_module error"),
            ),
        }
    }
}

fn params_to_string(params: &ModParams) -> String {
    let mut ret = String::new();
    for (name, pval) in params {
        if !ret.is_empty() {
            ret += " ";
        }
        ret += name;
        let txtval = pval.to_string();
        if !txtval.is_empty() {
            ret += "=";
            ret += &txtval;
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_to_string() {
        {
            let p0 = ModParams::new();
            assert_eq!(params_to_string(&p0), "".to_owned());
        }
        {
            let mut p1 = ModParams::new();
            p1.insert("foo".to_owned(), ModParamValue::Bool(true));
            assert_eq!(params_to_string(&p1), "foo=true".to_owned());
        }
        {
            let mut p2 = ModParams::new();
            p2.insert("bar".to_owned(), ModParamValue::Int(-17));
            assert_eq!(params_to_string(&p2), "bar=-17".to_owned());
        }
        {
            let mut p3 = ModParams::new();
            p3.insert("bar".to_owned(), ModParamValue::Int(42));
            p3.insert("foo".to_owned(), ModParamValue::Bool(true));
            assert_eq!(params_to_string(&p3), "bar=42 foo=true".to_owned());
        }
        {
            let mut p4 = ModParams::new();
            p4.insert("bar".to_owned(), ModParamValue::Int(42));
            p4.insert("foo".to_owned(), ModParamValue::Str("quz".to_string()));
            assert_eq!(params_to_string(&p4), "bar=42 foo=quz".to_owned());
        }
        {
            let mut p5 = ModParams::new();
            p5.insert("bar".to_owned(), ModParamValue::Int(42));
            p5.insert(
                "foo".to_owned(),
                ModParamValue::Array(vec![ModParamValue::Int(42), ModParamValue::Int(99)]),
            );
            assert_eq!(params_to_string(&p5), "bar=42 foo=42,99".to_owned());
        }
    }
}
