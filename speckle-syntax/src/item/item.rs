use syn::{
    ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait,
    ItemUnion,
};

pub enum Item {
    Static(ItemStatic),
    Const(ItemConst),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Union(ItemUnion),
    Fn(ItemFn),
    Trait(ItemTrait),
    Impl(ItemImpl),
    Macro(ItemMacro),
    Mod(ItemMod),
}
