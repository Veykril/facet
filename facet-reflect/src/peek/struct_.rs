use facet_core::{Field, FieldError, FieldFlags, StructType};

use crate::Peek;
use alloc::{vec, vec::Vec};

/// Lets you read from a struct (implements read-only struct operations)
#[derive(Clone, Copy)]
pub struct PeekStruct<'mem, 'facet, 'shape> {
    /// the underlying value
    pub(crate) value: Peek<'mem, 'facet, 'shape>,

    /// the definition of the struct!
    pub(crate) ty: StructType<'shape>,
}

impl core::fmt::Debug for PeekStruct<'_, '_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PeekStruct").finish_non_exhaustive()
    }
}

impl<'mem, 'facet, 'shape> PeekStruct<'mem, 'facet, 'shape> {
    /// Returns the struct definition
    #[inline(always)]
    pub fn ty(&self) -> &StructType {
        &self.ty
    }

    /// Returns the number of fields in this struct
    #[inline(always)]
    pub fn field_count(&self) -> usize {
        self.ty.fields.len()
    }

    /// Returns the value of the field at the given index
    #[inline(always)]
    pub fn field(&self, index: usize) -> Result<Peek<'mem, 'facet, 'shape>, FieldError> {
        self.ty
            .fields
            .get(index)
            .map(|field| unsafe {
                let field_data = self.value.data().field(field.offset);
                Peek::unchecked_new(field_data, field.shape())
            })
            .ok_or(FieldError::IndexOutOfBounds {
                index,
                bound: self.ty.fields.len(),
            })
    }

    /// Gets the value of the field with the given name
    #[inline]
    pub fn field_by_name(&self, name: &str) -> Result<Peek<'mem, 'facet, 'shape>, FieldError> {
        for (i, field) in self.ty.fields.iter().enumerate() {
            if field.name == name {
                return self.field(i);
            }
        }
        Err(FieldError::NoSuchField)
    }
}

impl<'mem, 'facet, 'shape> HasFields<'mem, 'facet, 'shape> for PeekStruct<'mem, 'facet, 'shape>
where
    'mem: 'facet,
{
    /// Iterates over all fields in this struct, providing both name and value
    #[inline]
    fn fields(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Field<'shape>, Peek<'mem, 'facet, 'shape>)> {
        (0..self.field_count()).filter_map(|i| {
            let field = self.ty.fields.get(i).copied()?;
            let value = self.field(i).ok()?;
            Some((field, value))
        })
    }
}

/// Trait for types that have field methods
///
/// This trait allows code to be written generically over both structs and enums
/// that provide field access and iteration capabilities.
pub trait HasFields<'mem, 'facet, 'shape>
where
    'mem: 'facet,
{
    /// Iterates over all fields in this type, providing both field metadata and value
    fn fields(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Field<'shape>, Peek<'mem, 'facet, 'shape>)>;

    /// Iterates over fields in this type that should be included when it is serialized
    fn fields_for_serialize(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Field<'shape>, Peek<'mem, 'facet, 'shape>)> {
        // This is a default implementation that filters out fields with `skip_serializing`
        // attribute and handles field flattening.
        self.fields()
            .filter(|(field, peek)| !unsafe { field.should_skip_serializing(peek.data()) })
            .flat_map(move |(mut field, peek)| {
                if field.flags.contains(FieldFlags::FLATTEN) {
                    let mut flattened = Vec::new();
                    if let Ok(struct_peek) = peek.into_struct() {
                        struct_peek
                            .fields_for_serialize()
                            .for_each(|item| flattened.push(item));
                    } else if let Ok(enum_peek) = peek.into_enum() {
                        // normally we'd serialize to something like:
                        //
                        //   {
                        //     "field_on_struct": {
                        //       "VariantName": { "field_on_variant": "foo" }
                        //     }
                        //   }
                        //
                        // But since `field_on_struct` is flattened, instead we do:
                        //
                        //   {
                        //     "VariantName": { "field_on_variant": "foo" }
                        //   }
                        field.name = enum_peek
                            .active_variant()
                            .expect("Failed to get active variant")
                            .name;
                        field.flattened = true;
                        flattened.push((field, peek));
                    } else {
                        // TODO: fail more gracefully
                        panic!("cannot flatten a {}", field.shape())
                    }
                    flattened
                } else {
                    vec![(field, peek)]
                }
            })
    }
}
