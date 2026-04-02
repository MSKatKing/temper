use temper_core::block_state_id::BlockStateId;
use crate::{PlacementContext, BLOCK_MAPPINGS};
use temper_core::pos::BlockPos;
use temper_world::World;

/// Macro to autogenerate the `BlockBehavior` trait and associated VTable structs.
///
/// This macro simply exists to make adding methods to blocks easier. It should NOT be used anywhere except in this file, and should also only be used once.
/// See below this macro for where to add functions.
///
/// The syntax for this macro is as follows: `fn <name>([mut]; <arguments>) [-> <return type>; <default return value>]`
/// - `name`: The name of the method
/// - `mut`: Optional, whether the function takes a mutable reference to the block or not
/// - `arguments`: Any additional arguments to the method
/// - `return type`: Optional, what the function returns
/// - `default return value`: Optional, required if return type is set, the default expression to use for blocks that do not implement this method.
macro_rules! block_behavior_trait {
    ($(fn $name:ident($($mut_meta:ident)?; $($argument:ident: $ty:ty),*) $(-> $ret:ty; $default:expr)?),* $(,)?) => {
        macro_rules! ptr_ret_ty {
            (mut) => {
                u32
            };
            () => {
                ()
            };
            (mut $retb:ty) => {
                (u32, $retb)
            };
            ($retb:ty) => {
                $retb
            };
        }

        macro_rules! id_ret_decode {
            (mut, $in_data:expr) => {
                ($in_data, ())
            };
            (, $in_data:expr) => {
                ((), ())
            };
            (mut $retb:ty, $in_data:expr) => {
                $in_data
            };
            ($retb:ty, $in_data:expr) => {
                ((), $retb)
            };
        }

        macro_rules! lambda_ret_ty {
            (mut; $data:expr;) => {
                $data
                    .try_into()
                    .unwrap_or_else(|_| panic!("Failed to convert block data back into id"))
            };
            () => {
                ()
            };
            (mut; $data:expr; $retb:ty) => {
                (
                    $data
                        .try_into()
                        .unwrap_or_else(|_| panic!("Failed to convert block data back into id")),
                    _ret,
                )
            };
            ($retb:ty) => {
                _ret
            };
        }

        pub trait BlockBehavior:
            TryInto<u32, Error = ()> + TryFrom<u32, Error = ()> + Clone + std::fmt::Debug
        {
            $(
                fn $name(&$($mut_meta)? self, $($argument: $ty),*) $(-> $ret)? { $($default)? }
            )*
        }

        #[allow(dead_code)]
        pub trait BlockDispatch {
            $(
                fn $name(&$($mut_meta)? self, $($argument: $ty),*) $(-> $ret)? { $($default)? }
            )*
        }

        pub struct BlockBehaviorTable {
            $(
                $name: fn(id: u32, $($argument: $ty),*) -> ptr_ret_ty!{$($mut_meta)?}
            ),*
        }

        impl BlockBehaviorTable {
            pub const fn from<T: BlockBehavior>() -> Self {
                Self {
                    $(
                        $name: |id, $($argument),*| {
                            let $($mut_meta)? data = T::try_from(id).unwrap_or_else(|_| panic!("Failed to convert id to data"));
                            let _ret = data.$name($($argument),*);
                            lambda_ret_ty!($($mut_meta; data;)? $($ret)?)
                        }
                    ),*
                }
            }
        }

        pub struct StateBehaviorTable {
            block: &'static BlockBehaviorTable,
            id: u32,
        }

        impl StateBehaviorTable {
            pub const fn spin_off(block: &'static BlockBehaviorTable, id: u32) -> Self {
                Self { block, id }
            }

            $(
                pub fn $name(&self, $($argument: $ty),*) -> ptr_ret_ty!{$($mut_meta)? $($ret)?} {
                    (self.block.$name)(self.id, $($argument),*)
                }
            )*
        }

        macro_rules! update_self {
            (mut, $s:expr, $new_id:expr) => {
                *$s = Self::new($new_id);
            };
            (, $s:expr, $new_id:expr) => {

            };
        }

        impl BlockDispatch for BlockStateId {
            $(
                fn $name(& $($mut_meta)? self, $($argument: $ty),*) $(-> $ret)? {
                    let (_new_id, _ret) = id_ret_decode!{$($mut_meta)? $($ret)?, BLOCK_MAPPINGS[self.raw() as usize].$name($($argument),*)};
                    update_self!{$($mut_meta)?, self, _new_id}
                    _ret
                }
            )*
        }
    };
}

// This is where methods are defined for blocks. See the macro above for the syntax.
//
// This is the only place where the `block_behavior_trait!` macro should be used.
block_behavior_trait!(
    fn get_placement_state(mut; _context: PlacementContext, _world: &World, _pos: BlockPos),
    fn update(mut; _world: &World, _pos: BlockPos),
    fn test(;),
);
