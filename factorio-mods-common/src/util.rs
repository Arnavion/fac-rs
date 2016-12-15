/// Deserializes a string or a sequence of strings into a vector of the target type.
pub fn deserialize_string_or_seq_string<T, D>(deserializer: &mut D) -> Result<Vec<T>, D::Error>
	where T: ::serde::Deserialize, D: ::serde::Deserializer {

	struct Visitor<T>(::std::marker::PhantomData<T>);

	impl<T> ::serde::de::Visitor for Visitor<T> where T: ::serde::Deserialize {
		type Value = Vec<T>;

		fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E> where E: ::serde::Error {
			let value = {
				let mut deserializer = StringNewTypeStructDeserializer(v, ::std::marker::PhantomData);
				::serde::Deserialize::deserialize(&mut deserializer)
			}.or_else(|_: E| {
				let mut deserializer = ::serde::de::value::ValueDeserializer::into_deserializer(v);
				::serde::Deserialize::deserialize(&mut deserializer)
			})?;
			Ok(vec![value])
		}

		fn visit_seq<V>(&mut self, mut visitor: V) -> ::std::result::Result<Self::Value, V::Error> where V: ::serde::de::SeqVisitor {
			let mut result: Vec<T> = vec![];

			while let Some(value) = visitor.visit()? {
				result.push(value);
			}

			visitor.end()?;

			Ok(result)
		}
	}

	deserializer.deserialize(Visitor(::std::marker::PhantomData))
}

struct StringNewTypeStructDeserializer<'a, E>(&'a str, ::std::marker::PhantomData<E>);

impl<'a, E> ::serde::Deserializer for StringNewTypeStructDeserializer<'a, E> where E: ::serde::Error {
	type Error = E;

	fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error> where V: ::serde::de::Visitor {
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_string<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error> where V: ::serde::de::Visitor {
		visitor.visit_str(self.0)
	}

	forward_to_deserialize! {
		bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str
		unit option seq seq_fixed_size bytes map unit_struct newtype_struct
		tuple_struct struct struct_field tuple enum ignored_any
	}
}
