macro_rules! impl_name_type {
    ($x:ident) => {
        impl std::fmt::Display for $x {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                let $x(ref name) = *self;
                name.fmt(f)
            }
        }

        impl AsRef<str> for $x {
            fn as_ref(&self) -> &str {
                let $x(ref name) = *self;
                name
            }
        }

        impl AsRef<String> for $x {
            fn as_ref(&self) -> &String {
                let $x(ref name) = *self;
                name
            }
        }

        impl<'a> From<&'a str> for $x {
            fn from(name: &str) -> Self {
                $x(name.to_string())
            }
        }

        impl From<String> for $x {
            fn from(name: String) -> Self {
                $x(name)
            }
        }

        impl From<$x> for String {
            fn from(name: $x) -> Self {
                let $x(name) = name;
                name
            }
        }

        impl serde::Serialize for $x {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: serde::Serializer
            {
                let $x(ref name) = *self;
                serializer.visit_str(name)
            }
        }

        impl serde::Deserialize for $x {
            fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                where D: serde::Deserializer
            {
                struct Visitor;

                impl serde::de::Visitor for Visitor {
                    type Value = $x;

                    fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                        where E: serde::de::Error
                    {
                        Ok($x(v.to_string()))
                    }

                    fn visit_string<E>(&mut self, v: String) -> Result<Self::Value, E>
                        where E: serde::de::Error
                    {
                        Ok($x(v))
                    }
                }

                deserializer.visit(Visitor)
            }
        }
    }
}
