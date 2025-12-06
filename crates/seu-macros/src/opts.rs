use syn::Attribute;
use syn::Error;
use syn::Ident;
use syn::Meta;
use syn::Path;
use syn::Token;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_quote;

enum OptionName {
    Crate,
}

impl Parse for OptionName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let peek = input.lookahead1();

        if peek.peek(Token![crate]) {
            let _: Token![crate] = input.parse()?;
            return Ok(Self::Crate);
        } else if peek.peek(Ident) {
            let ident: Ident = input.parse()?;
            let method = match ident.to_string().to_lowercase().as_str() {
                "name" => Self::Crate,
                other => {
                    return Err(Error::new(
                        ident.span(),
                        format!("Unknown option name {other}"),
                    ));
                }
            };

            return Ok(method);
        }

        Err(peek.error())
    }
}

enum OptionValue {
    Crate(Path),
}

impl Parse for OptionValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: OptionName = input.parse()?;
        let _: Token![=] = input.parse()?;

        match name {
            OptionName::Crate => {
                let path: Path = input.parse()?;
                Ok(Self::Crate(path))
            }
        }
    }
}

/// Options for the derive macro
pub struct Options {
    /// Target crate path for name resolution
    pub target_crate: Path,
}

impl Options {
    /// Load options from attributes applied to the deriver input data type.
    pub fn load_from(&mut self, attrs: &mut Vec<Attribute>) -> Result<(), Error> {
        let dt_attrs = attrs.extract_if(.., |item| {
            let Meta::List(meta) = &item.meta else {
                return false;
            };
            meta.path.is_ident("datatype")
        });

        for attr in dt_attrs {
            let Meta::List(meta) = attr.meta else {
                continue;
            };

            match syn::parse2(meta.tokens)? {
                OptionValue::Crate(path) => {
                    self.target_crate = path;
                }
            }
        }

        Ok(())
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            target_crate: parse_quote!(::seu),
        }
    }
}
