# Temper Blocks

This is a quick tutorial on how to use this library to create and modify block behavior and structs.

## Tips and Notes

- If the build script is emitting a warning about an unknown block, see the **Adding / Modifying Block Structs** section for more information.

## Creating Block Behavior

To create a new function for blocks to implement, navigate to the `behavior_trait.rs` file. Due to the complexity of the inner workings of the system, a macro has been built to make creating functions for blocks super easy. The macro syntax is pretty simple:

```rust
block_behavior_trait!(
    fn <function name>([mut]; <arguments>) [-> <return type>; {<default return value}],
    ...
);
```

- Function Name: The name of the function.
- mut: Optional; whether the function should allow blocks to be mutated. Note: the semicolon after mut is required whether mut is specified or not.
- Arguments: Any amount of arguments for the function to take in.
- Return Type: Optional; The return type of the function.
- Default Return Value: Only present if a return type is specified; the value returned by the default trait implementation of the function.

You can look at the existing functions for tips on usage as well. Simply add your function to this list for it to be a function that blocks can implement.

**PLEASE NOTE:** This macro should ***NOT*** be used anywhere else in the project. It is only here to autogenerate structs and function pointer types that are needed for function dispatch while making it easy to create and modify block functions.

## Implementing / Modifying Block Behavior

To implement or modify the behavior of a block, navigate to where the `BlockBehavior` trait is implemented for that block type.

**NOTE:** Not all blocks have their own block type/struct. Most blocks are lumped in to a single struct (for example, all slab blocks are implemented as the SlabBlock type). This is also how Minecraft implements this (blocks of the same type and logic are instantiated as the same class). Blocks can still be distinguished by the enum attached to the struct.

Once you find the `BlockBehavior` implementation, simply add or modify the function you want.

## Adding / Modifying Block Structs

To add a struct for a specific block type you must edit the `build_config.toml` file. Which section you edit is dependent on what you'd like to do. All blocks are placed into a struct no matter what, so you must either override the name or explicitly place the specific block type into a separate struct.

The `name_overrides` section is for assigning a name to a particular arrangement of block state properties. For example, slabs in the game have the block state properties `type` and `waterlogged`. The name `SlabBlock` was assigned to all blocks with that arrangement of block state properties.

**NOTE:** There are quite a few blocks that have completely different behavior to other blocks but share the same block state property configuration. In this situation, the `block_overrides` section may have to be used.

The `block_overrides` section is for placing specific blocks into different structs. If a block is placed here it will ignore any name given to its configuration of block state properties and use this name instead.

### Important Note

All block struct names **MUST** be unique. The generated crate will not compile otherwise.