/// Deserializes a string or a sequence of strings into a vector of the target type.
pub fn deserialize_string_or_seq_string<T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
	where T: ::serde::Deserialize, D: ::serde::Deserializer {

	struct Visitor<T>(::std::marker::PhantomData<T>);

	impl<T> ::serde::de::Visitor for Visitor<T> where T: ::serde::Deserialize {
		type Value = Vec<T>;

		fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
			formatter.write_str("a string or sequence of strings")
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: ::serde::de::Error {
			let value = {
				let deserializer = StringNewTypeStructDeserializer(v, ::std::marker::PhantomData);
				::serde::Deserialize::deserialize(deserializer)
			}.or_else(|_: E| {
				let deserializer = ::serde::de::value::ValueDeserializer::into_deserializer(v);
				::serde::Deserialize::deserialize(deserializer)
			})?;
			Ok(vec![value])
		}

		fn visit_seq<V>(self, mut visitor: V) -> ::std::result::Result<Self::Value, V::Error> where V: ::serde::de::SeqVisitor {
			let mut result: Vec<T> = vec![];

			while let Some(value) = visitor.visit()? {
				result.push(value);
			}

			Ok(result)
		}
	}

	deserializer.deserialize(Visitor(::std::marker::PhantomData))
}

struct StringNewTypeStructDeserializer<'a, E>(&'a str, ::std::marker::PhantomData<E>);

impl<'a, E> ::serde::Deserializer for StringNewTypeStructDeserializer<'a, E> where E: ::serde::de::Error {
	type Error = E;

	fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: ::serde::de::Visitor {
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: ::serde::de::Visitor {
		visitor.visit_str(self.0)
	}

	forward_to_deserialize! {
		bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str
		unit option seq seq_fixed_size bytes map unit_struct newtype_struct
		tuple_struct struct struct_field tuple enum ignored_any byte_buf
	}
}
