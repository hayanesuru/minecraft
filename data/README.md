# minecraft_data

minecraft version:
`1.21.4`

# Usage

```toml
[dependencies]
minecraft_data = "2.0"
```

```rust
pub use minecraft_data::*;

fn main() {
    assert_eq!(core::mem::size_of::<block_state>(), 2);
    assert_eq!(core::mem::size_of::<block>(), 2);

    let offset = oak_log::new().with_axis(prop_axis_x_y_z::z).encode() as raw_block_state;
    let state: block_state = block_state::new(block::oak_log.state_index() + offset).unwrap();
    assert_eq!(
        oak_log::decode((state.id() - block::oak_log.state_index()) as _).axis(),
        prop_axis_x_y_z::z
    );
    let state: block_state = encode_state!(oak_log(oak_log::new().with_axis(prop_axis_x_y_z::z)));
    assert_eq!(decode_state!(oak_log(state)).axis(), prop_axis_x_y_z::z);
    assert_eq!(state.to_fluid().to_fluid(), fluid::empty);

    let x = block::mud.state_default();
    let b = x.to_block();
    assert_eq!(b.name(), "mud");
    assert_eq!(Some(b), block::parse(b"mud"));

    assert_eq!(x.side_solid_full(), Some(0b111111));
    assert_eq!(x.side_solid_rigid(), Some(0b111111));
    assert_eq!(x.side_solid_center(), Some(0b111111));
    assert_eq!(x.full_cube(), Some(false));
}
```
