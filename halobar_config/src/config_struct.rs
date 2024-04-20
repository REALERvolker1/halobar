/// toml_edit really doesn't like it when I have required values. This macro is a shitty workaround.
///
/// generates `StructNameConfig` and `StructNameKnown` structs.
/// `StructNameConfig` is full of `Option<T>`s that get converted to `T`s in `StructNameKnown::hydrate`.
/// ```rs
/// config_struct! {
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
    ($( @known {$( $known_derive:path ),+} )? [$struct_name:ident] $( $( $( @conf #[$conf_meta:meta] (#[$conf_opt_meta:meta]) )? @conf $conf_key:ident: $conf_mod:path => $conf_name:tt, )* $(@reg #[$meta:meta] (#[$opt_meta:meta]) )? $name:ident: $type:ty = $default:expr ),+$(,)?) => {
        ::paste::paste! {
            #[derive(Debug, ::serde::Serialize, ::serde::Deserialize $($(, $known_derive )+)?)]
            pub struct [<$struct_name Known>] {
                $(
                    $(
                        $( #[$conf_meta] )?
                        pub $conf_key: $conf_mod::[<$conf_name Known>],
                    )*
                    $( #[$meta] )?
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

            #[derive(Debug, Default, ::serde::Serialize, ::serde::Deserialize)]
            pub struct [<$struct_name Config>] {
                $(
                    $(
                        $( #[$conf_opt_meta] )?
                        pub $conf_key: $conf_mod::[<$conf_name Config>],
                    )*
                    $( #[$opt_meta] )?
                    pub $name: Option<$type>,
                )+
            }

            impl [<$struct_name Config>] {
                /// Convert into a Known-type, applying the default values if need be.
                pub fn hydrate(self) -> [<$struct_name Known>] {
                    [<$struct_name Known>]::overlay(self)
                }
            }
        }
    };
}
