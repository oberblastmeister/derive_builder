use syn::Result;
use crate::ast::{Struct, Input};

impl Input<'_> {
    pub fn validate(&self) -> Result<()> {
        match self {
            Input::Struct(input) => input.validate(),
        }
    }
}

impl Struct<'_> {
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}
