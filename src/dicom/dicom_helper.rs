use dicom_object::InMemDicomObject;

#[macro_export]
macro_rules! define_dicom_struct {
    // Main macro to define a struct with fields, types, DICOM tags, and optionality
    ($name:ident, { $(($field_name:ident, $field_type:ty, $dicom_tag:expr, $is_optional:tt)),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            // Generate struct fields based on optionality
            $(
                pub $field_name: $crate::define_dicom_struct!(@optional $field_type, $is_optional),
            )*
        }

        impl $name {
            // Constructor function to create struct instances
            pub fn new($($field_name: $crate::define_dicom_struct!(@constructor_type $field_type, $is_optional)),*) -> Self {
                $name {
                    $(
                        $field_name,
                    )*
                }
            }

            // Function to format DICOM tags and their corresponding values into a String
            pub fn format_tags(&self) -> String {
                let mut result = String::new();
                $(
                    $crate::define_dicom_struct!(@to_string $field_name, $field_type, $dicom_tag, $is_optional, self, result);
                )*
                result
            }
        }
    };

    // Helper rule to wrap type in Option if the field is optional
    (@optional $field_type:ty, true) => {
        Option<$field_type>
    };
    (@optional $field_type:ty, false) => {
        $field_type
    };

    // Helper rule for constructor argument types
    (@constructor_type $field_type:ty, true) => {
        Option<$field_type>
    };
    (@constructor_type $field_type:ty, false) => {
        $field_type
    };

    // Helper rule to handle formatting for optional fields
    (@to_string $field_name:ident, $field_type:ty, $dicom_tag:expr, true, $self:ident, $result:ident) => {
        let value = match &$self.$field_name {
            Some(val) => format!("{}: Some({:?})\n", $dicom_tag, val),  // Directly use `val` (which is `String`)
            None => format!("{}: None (Optional)\n", $dicom_tag),
        };
        $result.push_str(value.as_str());
    };

    // Helper rule to handle formatting for mandatory fields
    (@to_string $field_name:ident, $field_type:ty, $dicom_tag:expr, false, $self:ident, $result:ident) => {
        $result.push_str(format!("{}: {:?}\n", $dicom_tag, &$self.$field_name).as_str());  // Borrow `String` as `&str`
    };
}

// Helper function to safely retrieve a tag value and convert it to a type T
pub fn get_value<T>(obj: &InMemDicomObject, tag: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    obj.element_by_name(tag)
        .ok()
        .and_then(|e| e.value().to_str().ok())
        .and_then(|v| v.parse::<T>().ok())
}