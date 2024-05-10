macro_rules! data_flags {
    ($struct_vis:vis $struct_name:ident => $enum:path { $( $name:ident => $enum_variant:ident ),+$(,)? }) => {
        /// The fields that the frontend is requesting, determined by a set of `FmtSegments`.
        ///
        /// TODO: Refactor these as bitflags
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $struct_vis struct $struct_name {
            $( pub $name: bool ),+
        }
        impl $struct_name {
            /// Join the true flags from two different sets into a single set, with all the sections that either set had enabled.
            pub fn merge_with(&mut self, other: Self) {
                $(
                    if other.$name {
                        self.$name = true;
                    }
                )+
            }
            ::paste::paste! {
            #[doc = "If any of the fields of " $struct_name "are enabled, this will return true. Useful for determining if we should skip entirely."]
            pub fn is_enabled(&self) -> bool {
                $( self.$name | )+ true
            }
            }
            /// Determine this from what the formatter is configured to show.
            pub fn from_segments(format_segments: ::halobar_config::fmt::FmtSegments<'_>) -> Self {
                let mut me = Self {
                    $( $name: false ),+
                };

                for segment in format_segments {
                    match segment {
                        ::halobar_config::fmt::Segment::Literal(_) => {}
                        ::halobar_config::fmt::Segment::Variable(var) => match var.ident.as_str() {
                            $( stringify!($name) => me.$name = true, )+
                            _ => {}
                        },
                    }
                }

                me
            }
            /// Determine whether the message is valid or invalid, from whether its corresponding flag is true.
            pub fn is_valid(&self, message: &$enum) -> bool {
                ::paste::paste! {
                    $(
                        const [<$name:upper _ENUM_VARIANT>]: $enum = $enum::$enum_variant;
                    )+

                    match message {
                        $( &[<$name:upper _ENUM_VARIANT>] => self.$name, )+
                    }
                }
            }
        }
    };
}
pub(crate) use data_flags;
