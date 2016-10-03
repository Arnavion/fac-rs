macro_rules! impl_deserialize_struct {
	(struct $struct_name:ident {
		$($field_name:ident: $field_type:ty,)*
	}) => {
		impl serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: serde::Deserializer {
				#[allow(non_camel_case_types)]
				enum Field {
					$($field_name,)*
					Unknown,
				}

				impl serde::Deserialize for Field {
					fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: serde::Deserializer {
						struct FieldVisitor;

						impl serde::de::Visitor for FieldVisitor {
							type Value = Field;

							fn visit_str<E>(&mut self, value: &str) -> Result<Field, E> where E: serde::Error {
								match value {
									$(stringify!($field_name) => Ok(Field::$field_name),)*
									_ => Ok(Field::Unknown),
								}
							}
						}

						deserializer.deserialize_struct_field(FieldVisitor)
					}
				}

				struct Visitor;

				impl serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: serde::de::MapVisitor {
						$(
							let mut $field_name: Option<$field_type> = None;
						)*

						while let Some(key) = try!(visitor.visit_key::<Field>()) {
							match key {
								$(
									Field::$field_name => {
										if $field_name.is_some() {
											return Err(<V::Error as serde::Error>::duplicate_field(stringify!($field_name)))
										}

										$field_name = Some(try!(visitor.visit_value()));
									}
								),*

								Field::Unknown => {
									try!(visitor.visit_value::<serde_json::Value>());
								},
							}
						}

						try!(visitor.end());

						$(
							let $field_name = match $field_name {
								Some(x) => x,
								None => try!(visitor.missing_field(stringify!($field_name)))
							};
						)*

						Ok($struct_name {
							$(
								$field_name: $field_name,
							)*
						})
					}
				}

				const FIELDS: &'static [&'static str] = &[$(stringify!($field_name),)*];
				deserializer.deserialize_struct(stringify!($struct_name), FIELDS, Visitor)
			}
		}
	}
}

macro_rules! impl_deserialize_u64 {
	($struct_name:ident) => {
		impl serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: serde::Deserializer {
				struct Visitor;

				impl serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_u64<E>(&mut self, v: u64) -> Result<Self::Value, E> where E: serde::Error {
						Ok($struct_name(v))
					}
				}

				deserializer.deserialize_u64(Visitor)
			}
		}
	};
}

#[macro_export]
macro_rules! make_deserializable {
	(struct $struct_name:ident {
		$($field_name:ident: $field_type:ty,)*
	}) => {
		#[derive(Debug)]
		struct $struct_name {
			$($field_name: $field_type),*
		}

		impl_deserialize_struct!(struct $struct_name {
			$($field_name: $field_type,)*
		});
	};

	(pub struct $struct_name:ident {
		$($field_name:ident: $field_type:ty,)*
	}) => {
		#[derive(Debug)]
		pub struct $struct_name {
			$($field_name: $field_type),*
		}

		impl_deserialize_struct!(struct $struct_name {
			$($field_name: $field_type,)*
		});
	};

	(struct $struct_name:ident(u64)) => {
		#[derive(Debug)]
		struct $struct_name(u64);

		impl_deserialize_u64!($struct_name);
	};

	(pub struct $struct_name:ident(u64)) => {
		#[derive(Debug)]
		pub struct $struct_name(u64);

		impl_deserialize_u64!($struct_name);
	};
}
