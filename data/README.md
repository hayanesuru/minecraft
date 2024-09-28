# minecraft_data

Supported version:  
`1.16.5`
`1.17.1`
`1.18.2`
`1.19.4`
`1.20.6`

# Usage

```toml
[dependencies]
minecraft_data = { version = "1.3", default-features = false, features = [
    "std",
    "1_16",
] }
```

```rust
pub use minecraft_data::v1_16::*;
pub use minecraft_data::{decode_state, encode_state};

fn main() {
    assert_eq!(core::mem::size_of::<block_state>(), 2);
    assert_eq!(core::mem::size_of::<block>(), 2);
    let offset = oak_log::new().with_axis(prop_axis_x_y_z::z).encode() as raw_block_state;
    let state: block_state = block_state::new(block::oak_log.state_index() + offset);
    assert_eq!(
        oak_log::decode((state.id() - block::oak_log.state_index()) as _).axis(),
        prop_axis_x_y_z::z
    );
    let state: block_state = encode_state!(oak_log(oak_log::new().with_axis(prop_axis_x_y_z::z)));
    assert_eq!(decode_state!(oak_log(state)).axis(), prop_axis_x_y_z::z);
    assert_eq!(state.to_fluid().to_fluid(), fluid::empty);
}
```
