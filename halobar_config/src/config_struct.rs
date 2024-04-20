/// toml_edit really doesn't like it when I have required values. This macro is a shitty workaround.
///
/// generates `StructNameConfig` and `StructNameKnown` structs.
/// `StructNameConfig` is full of `Option<T>`s that get converted to `T`s in `StructNameKnown::overlay`.
///
/// ```rust no_run
/// config_struct! {
///     @known {Clone}  // extra derives for the known type
///     @config {Clone, PartialEq}   // extra derives for the config type
///     [Window]
///     inner_spacing: i32 = 10,
///     border_width: u32 = 10,
///
///     // Additionally, you can nest `config_struct!`s and it "just works"
///     @conf window: crate::frontend::frontend_config => Window,
///     @conf log: crate::preinit::log => Log,
///
///     layer: Layer = Layer::Overlay,
///     exclusive: bool = true,
///
///     edge: ScreenEdge = ScreenEdge::Bottom,
/// }
/// ```
#[macro_export]
macro_rules! config_struct {
    ($( @known {$( $known_derive:path ),+} )? $( @config {$( $config_derive:path ),+} )? [$struct_name:ident] $( $( $( @conf $( @known #[$conf_known_meta:meta] )* $( @config #[$conf_config_meta:meta] )* )? @conf $conf_key:ident: $conf_mod:path => $conf_name:tt, )* $( @known #[$known_meta:meta] )* $( @config #[$config_meta:meta] )* $name:ident: $type:ty = $default:expr ),+$(,)?) => {
        ::paste::paste! {
            #[derive(Debug, ::serde::Serialize, ::serde::Deserialize $($(, $known_derive )+)?)]
            pub struct [<$struct_name Known>] {
                $(
                    $(
                        $($( #[$conf_known_meta] )*)?
                        pub $conf_key: $conf_mod::[<$conf_name Known>],
                    )*
                    $( #[$known_meta] )*
                    pub $name: $type,
                )+
            }
            impl Default for [<$struct_name Known>] {
                fn default() -> Self {
                    Self {
                        $(
                            $( $conf_key: $conf_mod::[<$conf_name Known>]::default(), )*
                            $name: $default,
                        )+
                    }
                }
            }
            impl [<$struct_name Known>] {
                /// Create a new instance of Self, using a partial-type and substituting any None values with the defaults.
                pub fn overlay(conf: [<$struct_name Config>]) -> Self {
                    Self {
                        $(
                            $(
                                $conf_key: $conf_mod::[<$conf_name Known>]::overlay(conf.$conf_key),
                            )*
                            $name: match conf.$name {
                                Some(c) => c,
                                None => $default,
                            },
                        )+
                    }
                }
                /// Convert into a partial, Config-type.
                pub fn into_wrapped(self) -> [<$struct_name Config>] {
                    [<$struct_name Config>] {
                        $(
                            $(
                                $conf_key: self.$conf_key.into_wrapped(),
                            )*
                            $name: Some(self.$name),
                        )+
                    }
                }
            }

            #[derive(Debug, Default, ::serde::Serialize, ::serde::Deserialize $($(, $config_derive )+)?)]
            pub struct [<$struct_name Config>] {
                $(
                    $(
                        $($( #[$conf_config_meta] )*)?
                        pub $conf_key: $conf_mod::[<$conf_name Config>],
                    )*
                    $( #[$config_meta] )*
                    pub $name: Option<$type>,
                )+
            }

            impl [<$struct_name Config>] {
                /// Convert into a Known-type, applying the default values if need be.
                pub fn into_known(self) -> [<$struct_name Known>] {
                    [<$struct_name Known>]::overlay(self)
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::config_struct;
    config_struct! {
        @known {Clone, Copy, PartialEq, Eq}
        @config {Clone, PartialEq}
        [Test]
        id: u8 = 0,
    }

    config_struct! {
        @known {Clone, PartialEq, Eq}
        [TestNest]
        @known #[serde(skip)]
        is_default: bool = true,
        @conf @config #[serde(flatten)]
        @conf test: super::test => Test,
        // due to syntax limitations, each nested config struct must be followed by a regular key-value.
        is_normal: &'static str = "maybe",
    }

    #[test]
    fn config_struct_merge() {
        let conf = TestNestConfig {
            is_default: Some(false),
            test: TestConfig { id: Some(69) },
            is_normal: None,
        };
        let conf_known = conf.into_known();

        let conf_definite = TestNestKnown {
            is_default: false,
            test: TestKnown { id: 69 },
            is_normal: "maybe",
        };

        assert_eq!(conf_known, conf_definite)
    }
}
