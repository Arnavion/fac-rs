//! Custom derives used by the factorio-mods-* crates.

#![recursion_limit = "200"]

#![deny(rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]

/// Derives `serde::Deserialize` on the newtype.
#[proc_macro_derive(newtype_deserialize)]
pub fn derive_newtype_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	run_derive(input, |ast| {
		let struct_name = &ast.ident;

		let ty = as_newtype(&ast).ok_or_else(||
			error(&ast, "#[derive(newtype_deserialize)] can only be used with tuple structs of one field"))?;

		let parser_func_name = syn::Ident::new(match identify_type(ty) {
			Some(Type::SemverVersion) => "parse_version",
			Some(Type::SemverVersionReq) => "parse_version_req",
			_ => return Err(error(&ty, "#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type")),
		}, proc_macro2::Span::call_site());

		let expecting_str = format!("a string that can be deserialized into a {}", struct_name);
		let error_str = format!("invalid {} {{:?}}: {{}}", struct_name);

		let result = quote::quote! {
			impl<'de> serde::Deserialize<'de> for #struct_name {
				fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: serde::Deserializer<'de> {
					struct Visitor;

					impl serde::de::Visitor<'_> for Visitor {
						type Value = #struct_name;

						fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
							formatter.write_str(#expecting_str)
						}

						fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E> where E: serde::de::Error {
							Ok(#struct_name(#parser_func_name(v).map_err(|err| serde::de::Error::custom(format!(#error_str, v, std::error::Error::description(&err))))?))
						}
					}

					deserializer.deserialize_any(Visitor)
				}
			}
		};

		Ok(result.into())
	})
}

/// Derives `std::fmt::Display` on the newtype.
#[proc_macro_derive(newtype_display)]
pub fn derive_newtype_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	run_derive(input, |ast| {
		let struct_name = &ast.ident;

		let ty = as_newtype(&ast).ok_or_else(||
			error(&ast, "#[derive(newtype_display)] can only be used with tuple structs of one field"))?;

		let result = match identify_type(ty) {
			Some(Type::SemverVersion) |
			Some(Type::SemverVersionReq) |
			Some(Type::String) |
			Some(Type::U64) => quote::quote! {
				impl std::fmt::Display for #struct_name {
					fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
						self.0.fmt(f)
					}
				}
			},

			None => return Err(error(&ty, "#[derive(newtype_display)] cannot be used with tuple structs with this wrapped type")),
		};

		Ok(result.into())
	})
}

/// Derives `std::str::FromStr` on the newtype.
#[proc_macro_derive(newtype_fromstr)]
pub fn derive_newtype_fromstr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	run_derive(input, |ast| {
		let struct_name = &ast.ident;

		let ty = as_newtype(&ast).and_then(identify_type);

		let result = match ty {
			Some(Type::String) => {
				quote::quote! {
					impl std::str::FromStr for #struct_name {
						type Err = std::string::ParseError;

						fn from_str(s: &str) -> Result<Self, Self::Err> {
							Ok(#struct_name(s.to_owned()))
						}
					}
				}
			},

			_ => return Err(error(&ast, "#[derive(newtype_fromstr)] can only be used with tuple structs of one String field")),
		};

		Ok(result.into())
	})
}

fn run_derive<F>(input: proc_macro::TokenStream, f: F) -> proc_macro::TokenStream where
	F: FnOnce(syn::DeriveInput) -> Result<proc_macro::TokenStream, syn::parse::Error>,
{
	match syn::parse(input).and_then(f) {
		Ok(token_stream) => token_stream,
		Err(err) => err.to_compile_error().into(),
	}
}

fn error<S, D>(spanned: &S, message: D) -> syn::parse::Error where S: syn::spanned::Spanned, D: std::fmt::Display {
	syn::parse::Error::new(spanned.span(), message)
}

fn as_newtype(ast: &syn::DeriveInput) -> Option<&syn::Type> {
	match ast.data {
		syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }), .. }) if unnamed.len() == 1 => Some(&unnamed[0].ty),
		_ => None,
	}
}

enum Type {
	SemverVersion,
	SemverVersionReq,
	String,
	U64,
}

fn identify_type(ty: &syn::Type) -> Option<Type> {
	let path =
		if let syn::Type::Path(syn::TypePath { qself: None, ref path }) = *ty {
			path
		}
		else {
			return None;
		};

	match *path {
		syn::Path { leading_colon: None, ref segments } if segments.len() == 2 => {
			let first_segment = &segments[0];
			let second_segment = &segments[1];

			if first_segment.arguments.is_empty() && first_segment.ident == "semver" && second_segment.arguments.is_empty() {
				match &*second_segment.ident.to_string() {
					"Version" => Some(Type::SemverVersion),
					"VersionReq" => Some(Type::SemverVersionReq),
					_ => None,
				}
			}
			else {
				None
			}
		},

		syn::Path { leading_colon: None, ref segments } if segments.len() == 1 => {
			let segment = &segments[0];

			match segment.arguments {
				syn::PathArguments::None => match &*segment.ident.to_string() {
					"String" => Some(Type::String),
					"u64" => Some(Type::U64),
					_ => None,
				},

				_ => None,
			}
		},

		_ => None,
	}
}
