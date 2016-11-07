#![crate_type = "proc-macro"]
#![feature(proc_macro, proc_macro_lib, slice_patterns)]
#![recursion_limit = "200"]

#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

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

				match identify_type(field_ty) {
					Ok(Type::String) => quote! {
						pub fn #field_name(&self) -> &str {
							&self.#field_name
						}
					},

					Ok(Type::VecString) => quote! {
						pub fn #field_name(&self) -> &[str] {
							&self.#field_name
						}
					},

					Ok(Type::Vec { ty }) => quote! {
						pub fn #field_name(&self) -> &[#ty] {
							&self.#field_name
						}
					},

					_ => quote! {
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
		#ast

		impl #struct_name {
			#(#getters)*
		}
	).to_string().parse().unwrap()
}

#[proc_macro_derive(newtype)]
pub fn derive_newtype(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let source = input.to_string();
	let ast = syn::parse_macro_input(&source).unwrap();

	let result = match ast.body {
		syn::Body::Struct(syn::VariantData::Tuple(ref fields)) if fields.len() == 1 => {
			let struct_name = &ast.ident;
			let field = &fields[0];
			let ty = &field.ty;

			match identify_type(ty).map_err(|_| format!("#[derive(newtype)] cannot be used for tuple structs with this wrapped type: {:?}", ty)).unwrap() {
				Type::SemverVersion => quote! {
					#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, newtype_deref, newtype_deserialize, newtype_display, newtype_new_if_public)]
					#ast
				},

				Type::SemverVersionReq => quote! {
					#[derive(Clone, Debug, PartialEq, newtype_deref, newtype_deserialize, newtype_display, newtype_new_if_public)]
					#ast
				},

				Type::String => quote! {
					#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, newtype_deref, newtype_deserialize, newtype_display, newtype_new_if_public)]
					#ast

					impl<T: ?Sized> AsRef<T> for #struct_name where str: AsRef<T> {
						fn as_ref(&self) -> &T {
							(&self.0 as &str).as_ref()
						}
					}
				},

				Type::U64 => quote! {
					#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, newtype_deref, newtype_deserialize, newtype_display, newtype_new_if_public)]
					#ast
				},

				Type::VecString => quote! {
					#[derive(Clone, Debug, newtype_deref, newtype_deserialize, newtype_new_if_public)]
					#ast
				},

				Type::Vec { .. } => quote! {
					#[derive(Clone, Debug, Deserialize, newtype_deref, newtype_new_if_public)]
					#ast
				},
			}
		},

		_ => panic!("#[derive(newtype)] can only be used with tuple structs of one field."),
	};

	quote!(#result).to_string().parse().unwrap()
}

#[proc_macro_derive(newtype_deref)]
pub fn derive_newtype_deref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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

	let result = match ast.body {
		syn::Body::Struct(syn::VariantData::Tuple(ref fields)) if fields.len() == 1 => {
			let struct_name = &ast.ident;
			let field = &fields[0];
			let ty = &field.ty;

			match identify_type(ty).map_err(|_| format!("#[derive(newtype_deref)] cannot be used with tuple structs with this wrapped type: {:?}", ty)).unwrap() {
				Type::SemverVersion => generate(struct_name, quote!(::semver::Version)),
				Type::SemverVersionReq => generate(struct_name, quote!(::semver::VersionReq)),
				Type::String => generate(struct_name, quote!(str)),
				Type::U64 => generate(struct_name, quote!(u64)),
				Type::VecString => generate(struct_name, quote!([String])),
				Type::Vec { ty } => generate(struct_name, quote!([#ty])),
			}
		},

		_ => panic!("#[derive(newtype_deref)] can only be used with tuple structs of one field."),
	};

	quote!(#ast #result).to_string().parse().unwrap()
}

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

					deserializer.deserialize_u64(Visitor)
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

					deserializer.deserialize_string(Visitor)
				}
			}
		}
	}

	let source = input.to_string();
	let ast = syn::parse_macro_input(&source).unwrap();

	let result = match ast.body {
		syn::Body::Struct(syn::VariantData::Tuple(ref fields)) if fields.len() == 1 => {
			let struct_name = &ast.ident;
			let field = &fields[0];
			let ty = &field.ty;

			match identify_type(ty).map_err(|_| format!("#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type: {:?}", ty)).unwrap() {
				Type::SemverVersion => {
					let result = generate_semver(struct_name, &TY_SEMVER_VERSION);
					quote!(#ast #result)
				},

				Type::SemverVersionReq => {
					let result = generate_semver(struct_name, &TY_SEMVER_VERSIONREQ);
					quote!(#ast #result)
				},

				Type::VecString => {
					let result = generate_string_or_seq_string(struct_name);
					quote!(#ast #result)
				},

				_ => quote!(#[derive(Deserialize)] #ast),
			}
		},

		_ => panic!("#[derive(newtype_deserialize)] can only be used with tuple structs of one field."),
	};

	quote!(#result).to_string().parse().unwrap()
}

#[proc_macro_derive(newtype_display)]
pub fn derive_newtype_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	fn generate(struct_name: &syn::Ident) -> quote::Tokens {
		quote! {
			impl ::std::fmt::Display for #struct_name {
				fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
					self.0.fmt(f)
				}
			}
		}
	}

	let source = input.to_string();
	let ast = syn::parse_macro_input(&source).unwrap();

	let result = match ast.body {
		syn::Body::Struct(syn::VariantData::Tuple(ref fields)) if fields.len() == 1 => {
			let struct_name = &ast.ident;
			let field = &fields[0];
			let ty = &field.ty;

			match identify_type(ty).map_err(|_| format!("#[derive(newtype_display)] cannot be used with tuple structs with this wrapped type: {:?}", ty)).unwrap() {
				Type::SemverVersion => generate(struct_name),
				Type::SemverVersionReq => generate(struct_name),
				Type::String => generate(struct_name),
				Type::U64 => generate(struct_name),
				_ => quote!(),
			}
		},

		_ => panic!("#[derive(newtype_display)] can only be used with tuple structs of one field."),
	};

	quote!(#ast #result).to_string().parse().unwrap()
}

#[proc_macro_derive(newtype_new_if_public)]
pub fn derive_newtype_new_if_public(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let source = input.to_string();
	let ast = syn::parse_macro_input(&source).unwrap();

	let result = if ast.vis == syn::Visibility::Public {
		quote!(#[derive(new)] #ast)
	}
	else {
		quote!(#ast)
	};

	quote!(#result).to_string().parse().unwrap()
}

lazy_static! {
	static ref TY_SEMVER_VERSION: ::syn::Ty = syn::parse_type("::semver::Version").unwrap();
	static ref TY_SEMVER_VERSIONREQ: ::syn::Ty = syn::parse_type("::semver::VersionReq").unwrap();
	static ref TY_STRING: ::syn::Ty = syn::parse_type("String").unwrap();
	static ref TY_U64: ::syn::Ty = syn::parse_type("u64").unwrap();
}

#[derive(Debug)]
enum Type<'a> {
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
		if ident.to_string() != "Vec" {
			return Err(())
		}

		if let syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData { ref types, .. }) = *parameters {
			let wrapped_ty = &types[0];
			if wrapped_ty == &*TY_STRING {
				Ok(Type::VecString)
			}
			else {
				Ok(Type::Vec { ty: &types[0] })
			}
		}
		else {
			unreachable!();
		}
	}
	else {
		Err(())
	}
}
