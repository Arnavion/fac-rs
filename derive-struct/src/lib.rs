#![crate_type = "proc-macro"]
#![feature(proc_macro, proc_macro_lib)]
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
	let ast = syn::parse_macro_input(&source).unwrap();
	let struct_name = &ast.ident;

	let getters = match ast.body {
		syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
			fields.iter().map(|field| {
				let field_name = &field.ident;
				let field_ty = &field.ty;

				let field_doc_attr = field.attrs.iter().filter_map(|attr| {
					match (&attr.value, &attr.is_sugared_doc) {
						(&syn::MetaItem::NameValue(ref ident, _), &true) if ident.to_string() == "doc" => Some(attr),
						_ => None,
					}
				}).next();

				match identify_type(field_ty) {
					Ok(Type::OptionString) => quote! {
						#field_doc_attr
						pub fn #field_name(&self) -> Option<&str> {
							self.#field_name.as_ref()
						}
					},

					Ok(Type::Option { ty }) => quote! {
						#field_doc_attr
						pub fn #field_name(&self) -> Option<&#ty> {
							self.#field_name.as_ref()
						}
					},

					Ok(Type::String) => quote! {
						#field_doc_attr
						pub fn #field_name(&self) -> &str {
							&self.#field_name
						}
					},

					Ok(Type::VecString) => quote! {
						#field_doc_attr
						pub fn #field_name(&self) -> &[str] {
							&self.#field_name
						}
					},

					Ok(Type::Vec { ty }) => quote! {
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
	).to_string().parse().unwrap()
}

/// Derives `serde::Deserialize` on the newtype.
#[proc_macro_derive(newtype_deserialize)]
pub fn derive_newtype_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	fn generate_semver(struct_name: &syn::Ident, semver_type: &syn::Ty) -> quote::Tokens {
		quote! {
			impl ::serde::Deserialize for #struct_name {
				fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
					struct Visitor;

					impl ::serde::de::Visitor for Visitor {
						type Value = #struct_name;

						fn visit_str<E>(&mut self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::Error {
							let version =
								if let Ok(version) = #semver_type::parse(v) {
									version
								}
								else {
									let fixed_version = fixup_version(v);
									#semver_type::parse(&fixed_version).map_err(|err| ::serde::Error::invalid_value(::std::error::Error::description(&err)))?
								};

							Ok(#struct_name(version))
						}
					}

					deserializer.deserialize(Visitor)
				}
			}
		}
	}

	fn generate_string_or_seq_string(struct_name: &syn::Ident) -> quote::Tokens {
		quote! {
			impl ::serde::Deserialize for #struct_name {
				fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
					struct Visitor;

					impl ::serde::de::Visitor for Visitor {
						type Value = #struct_name;

						fn visit_str<E>(&mut self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::Error {
							Ok(#struct_name(vec![v.to_string()]))
						}

						fn visit_seq<V>(&mut self, mut visitor: V) -> ::std::result::Result<Self::Value, V::Error> where V: ::serde::de::SeqVisitor {
							let mut result: Vec<String> = vec![];

							while let Some(value) = visitor.visit()? {
								result.push(value);
							}

							visitor.end()?;

							Ok(#struct_name(result))
						}
					}

					deserializer.deserialize(Visitor)
				}
			}
		}
	}

	let source = input.to_string();
	let ast = syn::parse_macro_input(&source).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).map_err(|_| format!("#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type: {:?}", ty)).unwrap() {
				Type::SemverVersion => {
					generate_semver(struct_name, &TY_SEMVER_VERSION)
				},

				Type::SemverVersionReq => {
					generate_semver(struct_name, &TY_SEMVER_VERSIONREQ)
				},

				Type::VecString => {
					generate_string_or_seq_string(struct_name)
				},

				_ => panic!("#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type: {:?}", ty),
			}
		},

		None => panic!("#[derive(newtype_deserialize)] can only be used with tuple structs of one field."),
	};

	result.to_string().parse().unwrap()
}

/// Derives `std::fmt::Display` on the newtype.
#[proc_macro_derive(newtype_display)]
pub fn derive_newtype_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let source = input.to_string();
	let ast = syn::parse_macro_input(&source).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).map_err(|_| format!("#[derive(newtype_display)] cannot be used with tuple structs with this wrapped type: {:?}", ty)).unwrap() {
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

	result.to_string().parse().unwrap()
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
	let ast = syn::parse_macro_input(&source).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).map_err(|_| format!("#[derive(newtype_ref)] cannot be used with tuple structs with this wrapped type: {:?}", ty)).unwrap() {
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
				Type::VecString => generate(struct_name, quote!([String])),
				Type::Vec { ty } => generate(struct_name, quote!([#ty])),
				_ => panic!("#[derive(newtype_ref)] cannot be used for tuple structs with this wrapped type: {:?}", ty),
			}
		},

		None => panic!("#[derive(newtype_ref)] can only be used with tuple structs of one field."),
	};

	result.to_string().parse().unwrap()
}

lazy_static! {
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
	OptionString,
	Option { ty: &'a syn::Ty },
	SemverVersion,
	SemverVersionReq,
	String,
	U64,
	VecString,
	Vec { ty: &'a syn::Ty },
}

fn identify_type<'a>(ty: &'a syn::Ty) -> Result<Type<'a>, ()> {
	if ty == &*TY_SEMVER_VERSION {
		Ok(Type::SemverVersion)
	}
	else if ty == &*TY_SEMVER_VERSIONREQ {
		Ok(Type::SemverVersionReq)
	}
	else if ty == &*TY_STRING {
		Ok(Type::String)
	}
	else if ty == &*TY_U64 {
		Ok(Type::U64)
	}
	else if let syn::Ty::Path(_, syn::Path { ref segments, .. }) = *ty {
		if segments.len() != 1 {
			return Err(())
		}

		let syn::PathSegment { ref ident, ref parameters } = segments[0];
		if let syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData { ref types, .. }) = *parameters {
			let ident = ident.to_string();
			if ident != "Option" && ident != "Vec" {
				return Err(());
			}

			let wrapped_ty = &types[0];
			if wrapped_ty == &*TY_STRING {
				match ident.as_ref() {
					"Option" => Ok(Type::OptionString),
					"Vec" => Ok(Type::VecString),
					_ => Err(()),
				}
			}
			else {
				match ident.as_ref() {
					"Option" => Ok(Type::Option { ty: wrapped_ty }),
					"Vec" => Ok(Type::Vec { ty: wrapped_ty }),
					_ => Err(()),
				}
			}
		}
		else {
			Err(())
		}
	}
	else {
		Err(())
	}
}
