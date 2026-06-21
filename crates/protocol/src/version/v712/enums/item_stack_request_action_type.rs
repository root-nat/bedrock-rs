// `CraftResults` keeps the `#[deprecated]` marker (it is Mojang's deprecated craft-results action),
// but its fields must still be (de)serialized by the generated ProtoCodec impl in this module, which
// would otherwise raise self-referential deprecation warnings.
#![allow(deprecated)]

use crate::ProtoVersion;
use crate::version::v712::types::ItemStackWithoutUserData;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum ItemStackRequestActionType<V: ProtoVersion> {
    Take {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
        destination: V::ItemStackRequestSlotInfo,
    } = 0,
    Place {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
        destination: V::ItemStackRequestSlotInfo,
    } = 1,
    Swap {
        source: V::ItemStackRequestSlotInfo,
        destination: V::ItemStackRequestSlotInfo,
    } = 2,
    Drop {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
        randomly: bool,
    } = 3,
    Destroy {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
    } = 4,
    Consume {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
    } = 5,
    Create {
        slot: i8,
    } = 6,
    #[deprecated]
    PlaceInItemContainer = 7,
    #[deprecated]
    TakeFromItemContainer = 8,
    ScreenLabTableCombine = 9,
    ScreenBeaconPayment {
        #[endianness(var)]
        primary_effect: i32,
        #[endianness(var)]
        secondary_effect: i32,
    } = 10,
    ScreenHUDMineBlock {
        #[endianness(var)]
        hotbar_slot: i32,
        #[endianness(var)]
        predicted_durability: i32,
        #[endianness(var)]
        stack_network_id: i32,
    } = 11,
    CraftRecipe {
        #[endianness(var)]
        recipe_network_id: i32,
        number_of_requested_crafts: i8,
    } = 12,
    CraftRecipeAuto {
        #[endianness(var)]
        recipe_network_id: i32,
        number_of_requested_crafts: i8,
        times_crafted: i8,
        #[vec_repr(u8)]
        ingredients: Vec<V::ItemDescriptorType>,
    } = 13,
    CraftCreative {
        // The client sends this as a plain UNSIGNED varint (gophertunnel's CreativeItemNetworkID is
        // uint32), so it must be decoded as `u32` — decoding as a signed/zigzag `i32` mangles it
        // (e.g. a wire value of 21 reads back as -11).
        #[endianness(var)]
        creative_item_network_id: u32,
        number_of_requested_crafts: i8,
    } = 14,
    CraftRecipeOptional {
        #[endianness(var)]
        recipe_network_id: i32,
        #[endianness(le)]
        filtered_strings_index: i32,
    } = 15,
    CraftRepairAndDisenchant {
        #[endianness(var)]
        recipe_network_id: i32,
        number_of_requested_crafts: i8,
        #[endianness(var)]
        repair_cost: i32,
    } = 16,
    CraftLoom {
        pattern_id: String,
        number_of_requested_crafts: i8,
    } = 17,
    #[deprecated = "Ask Tylaing"]
    CraftNonImplemented = 18,
    #[deprecated = "Ask Tylaing"]
    CraftResults {
        // Wire (PMMP CraftResultsDeprecatedStackRequestAction / gophertunnel): an unsigned-varint
        // count, then `count` ItemStackWithoutUserData entries, then a single byte (times_crafted).
        // These fields MUST be read — leaving them out under-reads the action and corrupts every
        // following action in the same ItemStackRequest.
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        result_items: Vec<ItemStackWithoutUserData>,
        times_crafted: u8,
    } = 19,
}
