use std::collections::HashMap;
use std::ptr::NonNull;

use crate::{trace, InitFieldSlot, ShapeDesc, Shapely};

enum Destination {
    /// Writes directly to an (uninitialized) struct field
    StructField { field_addr: Option<NonNull<u8>> },

    /// Inserts into a HashMap
    HashMap { map: NonNull<u8>, key: String },
}

/// Allows writing into a struct field or inserting into a hash map.
pub struct Slot<'s> {
    /// where to write the value
    dest: Destination,

    /// shape of the field / hashmap value we're writing
    field_shape: ShapeDesc,

    /// lifetime marker
    init_field_slot: InitFieldSlot<'s>,
}

impl<'s> Slot<'s> {
    #[inline(always)]
    pub fn for_struct_field(
        field_addr: Option<NonNull<u8>>,
        field_shape: ShapeDesc,
        init_field_slot: InitFieldSlot<'s>,
    ) -> Self {
        Self {
            dest: Destination::StructField { field_addr },
            field_shape,
            init_field_slot,
        }
    }

    #[inline(always)]
    pub fn for_hash_map(
        map: NonNull<u8>,
        field_shape: ShapeDesc,
        key: String,
        init_field_slot: InitFieldSlot<'s>,
    ) -> Self {
        Self {
            dest: Destination::HashMap { map, key },
            field_shape,
            init_field_slot,
        }
    }

    pub fn fill<T: Shapely>(mut self, value: T) {
        if self.field_shape != T::shape_desc() {
            panic!(
                "Attempted to fill a field with an incompatible shape.\n\
                Expected shape: \x1b[33m{:?}\x1b[0m\n\
                Actual shape: \x1b[33m{:?}\x1b[0m\n\
                This is undefined behavior and we're refusing to proceed.",
                self.field_shape.get(),
                T::shape()
            );
        }
        trace!(
            "Filling slot with value of type: \x1b[33m{}\x1b[0m",
            std::any::type_name::<T>()
        );

        unsafe {
            match self.dest {
                Destination::StructField { field_addr } => {
                    if self.init_field_slot.is_init() {
                        trace!("Field already initialized, dropping existing value");
                        if let Some(drop_fn) = self.field_shape.get().drop_in_place {
                            drop_fn(
                                field_addr
                                    .map(|p| p.as_ptr())
                                    .unwrap_or(NonNull::dangling().as_ptr()),
                            );
                        }
                    }
                    if let Some(field_addr) = field_addr {
                        let field_addr = field_addr.as_ptr();
                        trace!(
                            "Filling struct field at address: \x1b[33m{:?}\x1b[0m",
                            field_addr
                        );
                        std::ptr::write(field_addr as *mut T, value);
                    } else {
                        // ZST, no need to do anything
                        trace!("Skipping write for ZST field");
                        std::mem::forget(value);
                    }
                }
                Destination::HashMap { map, key } => {
                    let map = &mut *(map.as_ptr() as *mut HashMap<String, T>);
                    trace!(
                        "Inserting value into HashMap with key: \x1b[33m{}\x1b[0m",
                        key
                    );
                    map.insert(key, value);
                }
            }
        }
        self.init_field_slot.mark_as_init();
    }

    pub fn fill_from_partial(mut self, partial: crate::Partial<'_>) {
        if self.field_shape != partial.shape_desc() {
            panic!(
                "Attempted to fill a field with an incompatible shape.\n\
                Expected shape: {:?}\n\
                Actual shape: {:?}\n\
                This is undefined behavior and we're refusing to proceed.",
                self.field_shape.get(),
                partial.shape_desc().get()
            );
        }

        unsafe {
            match self.dest {
                Destination::StructField { field_addr } => {
                    let size = self.field_shape.get().layout.size();
                    if self.init_field_slot.is_init() {
                        if let Some(drop_fn) = self.field_shape.get().drop_in_place {
                            drop_fn(
                                field_addr
                                    .map(|p| p.as_ptr())
                                    .unwrap_or(std::ptr::null_mut()),
                            );
                        }
                    }
                    if let Some(field_addr) = field_addr {
                        let field_addr = field_addr.as_ptr();
                        trace!(
                            "Filling struct field: src=\x1b[33m{:?}\x1b[0m, dst=\x1b[33m{:?}\x1b[0m, size=\x1b[33m{}\x1b[0m bytes",
                            partial.addr.unwrap().as_ptr(),
                            field_addr,
                            size
                        );
                        partial.move_into(field_addr);
                    } else {
                        trace!("Skipping write for ZST field");
                        drop(partial)
                    }
                }
                Destination::HashMap { map: _, ref key } => {
                    trace!(
                        "Filling HashMap entry: key=\x1b[33m{}\x1b[0m, src=\x1b[33m{:?}\x1b[0m, size=\x1b[33m{}\x1b[0m bytes",
                        key,
                        partial.addr.unwrap().as_ptr(),
                        self.field_shape.get().layout.size()
                    );
                    // TODO: Implement for HashMap
                    // I guess we need another field in the vtable?
                    panic!("fill_from_partial not implemented for HashMap");
                }
            }
        }
        self.init_field_slot.mark_as_init();
    }
}
