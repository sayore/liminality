# Prepend module docs
cat << 'DOC_EOF' > temp.rs
//! The pure data model for Liminality.
//! This crate is the canonical representation of a simulated logistics world.

DOC_EOF
cat crates/liminality-model/src/lib.rs >> temp.rs
mv temp.rs crates/liminality-model/src/lib.rs

# We can also add some doc comments to important structs across different files.
