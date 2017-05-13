#![crate_type = "proc-macro"]
#![recursion_limit = "200"]

//! A helper crate for easily deriving structs.

#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

/// Generates getters for every field of a non-tuple struct.
#[proc_macro_derive(getters)]
pub fn derive_getters(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let source = input.to_string();
	let ast = syn::parse_derive_input(&source).unwrap();
	let struct_name = &ast.ident;

	let getters = match ast.body {
		syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
			fields.iter().map(|field| {
				let field_name = &field.ident;
				let field_ty = &field.ty;

				let field_doc_attr = field.attrs.iter().filter_map(|attr| {
					match &attr.value {
						&syn::MetaItem::NameValue(ref ident, _) if ident.to_string() == "doc" => Some(attr),
						_ => None,
					}
				}).next();

				match identify_type(field_ty) {
					Some(Type::Bool) => quote! {
						#field_doc_attr
						pub fn #field_name(&self) -> bool {
							self.#field_name
						}
					},

					Some(Type::Option { ty }) => quote! {
						#field_doc_attr
						pub fn #field_name(&self) -> Option<&#ty> {
							self.#field_name.as_ref()
						}
					},

					Some(Type::String) => quote! {
						#field_doc_attr
						pub fn #field_name(&self) -> &str {
							&self.#field_name
						}
					},

					Some(Type::Vec { ty }) => quote! {
						#field_doc_attr
						pub fn #field_name(&self) -> &[#ty] {
							&self.#field_name
						}
					},

					_ => quote! {
						#field_doc_attr
						pub fn #field_name(&self) -> &#field_ty {
							&self.#field_name
						}
					},
				}
			})
		},

		_ => panic!("#[derive(getters)] can only be used with non-tuple structs."),
	};

	quote!(
		impl #struct_name {
			#(#getters)*
		}
	).parse().unwrap()
}

/// Derives `serde::Deserialize` on the newtype.
#[proc_macro_derive(newtype_deserialize)]
pub fn derive_newtype_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	fn generate_semver(struct_name: &syn::Ident, parser_func_name: quote::Tokens) -> quote::Tokens {
		let expecting_str = format!("a string that can be deserialized into a {}", struct_name);
		let error_str = format!("invalid {} {{:?}}: {{}}", struct_name);

		quote! {
			impl<'de> ::serde::Deserialize<'de> for #struct_name {
				fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer<'de> {
					struct Visitor;

					impl<'de> ::serde::de::Visitor<'de> for Visitor {
						type Value = #struct_name;

						fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
							formatter.write_str(#expecting_str)
						}

						fn visit_str<E>(self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::de::Error {
							Ok(#struct_name(#parser_func_name(v).map_err(|err| ::serde::de::Error::custom(format!(#error_str, v, ::std::error::Error::description(&err))))?))
						}
					}

					deserializer.deserialize_any(Visitor)
				}
			}
		}
	}

	let source = input.to_string();
	let ast = syn::parse_derive_input(&source).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).ok_or_else(|| format!("#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type: {:?}", ty)).unwrap() {
				Type::SemverVersion => generate_semver(struct_name, quote!(parse_version)),

				Type::SemverVersionReq => generate_semver(struct_name, quote!(parse_version_req)),

				_ => panic!("#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type: {:?}", ty),
			}
		},

		None => panic!("#[derive(newtype_deserialize)] can only be used with tuple structs of one field."),
	};

	result.parse().unwrap()
}

/// Derives `std::fmt::Display` on the newtype.
#[proc_macro_derive(newtype_display)]
pub fn derive_newtype_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let source = input.to_string();
	let ast = syn::parse_derive_input(&source).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).ok_or_else(|| format!("#[derive(newtype_display)] cannot be used with tuple structs with this wrapped type: {:?}", ty)).unwrap() {
				Type::SemverVersion |
				Type::SemverVersionReq |
				Type::String |
				Type::U64 => quote! {
					impl ::std::fmt::Display for #struct_name {
						fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
							self.0.fmt(f)
						}
					}
				},
				_ => panic!("#[derive(newtype_display)] cannot be used with tuple structs with this wrapped type: {:?}", ty),
			}
		},

		None => panic!("#[derive(newtype_display)] can only be used with tuple structs of one field."),
	};

	result.parse().unwrap()
}

/// Derives `std::ops::Deref` and `std::ops::DerefMut` on the newtype.
#[proc_macro_derive(newtype_ref)]
pub fn derive_newtype_ref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	fn generate(struct_name: &syn::Ident, wrapped_type: quote::Tokens) -> quote::Tokens {
		quote! {
			impl ::std::ops::Deref for #struct_name {
				type Target = #wrapped_type;

				fn deref(&self) -> &Self::Target {
					&self.0
				}
			}

			impl ::std::ops::DerefMut for #struct_name {
				fn deref_mut(&mut self) -> &mut Self::Target {
					&mut self.0
				}
			}
		}
	}

	let source = input.to_string();
	let ast = syn::parse_derive_input(&source).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).ok_or_else(|| format!("#[derive(newtype_ref)] cannot be used with tuple structs with this wrapped type: {:?}", ty)).unwrap() {
				Type::SemverVersion => generate(struct_name, quote!(::semver::Version)),
				Type::SemverVersionReq => generate(struct_name, quote!(::semver::VersionReq)),
				Type::String => {
					let deref = generate(struct_name, quote!(str));

					quote! {
						#deref

						impl<T: ?Sized> AsRef<T> for #struct_name where str: AsRef<T> {
							fn as_ref(&self) -> &T {
								(&self.0 as &str).as_ref()
							}
						}
					}
				},
				Type::U64 => generate(struct_name, quote!(u64)),
				Type::Vec { ty } => generate(struct_name, quote!([#ty])),
				_ => panic!("#[derive(newtype_ref)] cannot be used for tuple structs with this wrapped type: {:?}", ty),
			}
		},

		None => panic!("#[derive(newtype_ref)] can only be used with tuple structs of one field."),
	};

	result.parse().unwrap()
}

lazy_static! {
	static ref TY_BOOL: ::syn::Ty = syn::parse_type("bool").unwrap();
	static ref TY_SEMVER_VERSION: ::syn::Ty = syn::parse_type("::semver::Version").unwrap();
	static ref TY_SEMVER_VERSIONREQ: ::syn::Ty = syn::parse_type("::semver::VersionReq").unwrap();
	static ref TY_STRING: ::syn::Ty = syn::parse_type("String").unwrap();
	static ref TY_U64: ::syn::Ty = syn::parse_type("u64").unwrap();
}

fn as_newtype(ast: &syn::MacroInput) -> Option<&syn::Ty> {
	match ast.body {
		syn::Body::Struct(syn::VariantData::Tuple(ref fields)) if fields.len() == 1 => Some(&fields[0].ty),
		_ => None,
	}
}

#[derive(Debug)]
enum Type<'a> {
	Bool,
	Option { ty: &'a syn::Ty },
	SemverVersion,
	SemverVersionReq,
	String,
	U64,
	Vec { ty: &'a syn::Ty },
}

fn identify_type<'a>(ty: &'a syn::Ty) -> Option<Type<'a>> {
	if ty == &*TY_BOOL {
		Some(Type::Bool)
	}
	else if ty == &*TY_SEMVER_VERSION {
		Some(Type::SemverVersion)
	}
	else if ty == &*TY_SEMVER_VERSIONREQ {
		Some(Type::SemverVersionReq)
	}
	else if ty == &*TY_STRING {
		Some(Type::String)
	}
	else if ty == &*TY_U64 {
		Some(Type::U64)
	}
	else if let syn::Ty::Path(_, syn::Path { ref segments, .. }) = *ty {
		if segments.len() != 1 {
			return None
		}

		let syn::PathSegment { ref ident, ref parameters } = segments[0];
		if let syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData { ref types, .. }) = *parameters {
			let ident = ident.to_string();
			if ident != "Option" && ident != "Vec" {
				return None;
			}

			let wrapped_ty = &types[0];
			match ident.as_ref() {
				"Option" => Some(Type::Option { ty: wrapped_ty }),
				"Vec" => Some(Type::Vec { ty: wrapped_ty }),
				_ => unreachable!(),
			}
		}
		else {
			None
		}
	}
	else {
		None
	}
}
