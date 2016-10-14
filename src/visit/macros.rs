
/// Define a trait as usual, and a macro that can be used to instantiate
/// implementations of it.
///
/// Well almost: There *must* be markers of `@section type`, `@section self`
/// and `@section self_ref` before the associated types, `self` methods
/// and `&self` methods respectively.
macro_rules! define_trait {
    ($(#[$doc:meta])* pub trait $name:ident $($methods:tt)*) => {
        macro_rules! $name {
            ($m:ident $extra:tt) => {
                $m! {
                    $extra
                    pub trait $name $($methods)*
                }
            }
        }
        $(#[$doc])*
        remove_sections! { [] 
            pub trait $name $($methods)*
        }
    }
}

macro_rules! remove_sections {
    ([$($stack:tt)*]) => {
        $($stack)*
    };
    ([$($stack:tt)*] @section $x:ident $($t:tt)*) => {
        remove_sections!([$($stack)*] $($t)*);
    };
    ([$($stack:tt)*] { $($tail:tt)* }) => {
        $($stack)* {
            remove_sections!([] $($tail)*);
        }
    };
    ([$($stack:tt)*] $t:tt $($tail:tt)*) => {
        remove_sections!([$($stack)* $t] $($tail)*);
    };
}

macro_rules! deref {
    ($e:expr) => (*$e);
}

/// Implement a trait by delegation. By default as if we are delegating
/// from &G to G.
macro_rules! delegate_impl {
    ([] $($rest:tt)*) => {
        delegate_impl! { [['a, G], G, &'a G, deref] $($rest)* }
    };
    ([[$($param:tt)*], $self_type:ident, $self_wrap:ty, $self_map:ident]
     pub trait $name:ident : $sup:ty {
        $(
        @section type
        $(
            $(#[$_attr1:meta])*
            type $assoc_name:ident : $bound:ty;
        )+
        )*
        $(
        @section self
        $(
            $(#[$_attr2:meta])*
            fn $fname_self:ident(self $(,$arg2:ident : $argty2:ty)*) -> $ret2:ty;
        )+
        )*
        $(
        @section self_ref
        $(
            $(#[$_attr3:meta])*
            fn $fname:ident(&self $(,$arg:ident : $argty:ty)*) -> $ret:ty;
        )+
        )*
    }) => {
        impl<$($param)*> $name for $self_wrap where $self_type: $name {
            $(
            $(
                type $assoc_name = $self_type::$assoc_name;
            )*
            )*
            $(
            $(
                fn $fname_self(self $(,$arg2: $argty2)*) -> $ret2 {
                    $self_map!(self).$fname_self($($arg2),*)
                }
            )*
            )*
            $(
            $(
                fn $fname(&self $(,$arg: $argty)*) -> $ret {
                    $self_map!(self).$fname($($arg),*)
                }
            )*
            )*
        }
    }
}

