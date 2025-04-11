use super::*;

/// Processes a regular struct to implement Facet
///
/// Example input:
/// ```rust
/// struct Blah {
///     foo: u32,
///     bar: String,
/// }
/// ```
pub(crate) fn process_struct(parsed: Struct) -> proc_macro::TokenStream {
    let struct_name = parsed.name.to_string();

    // Generate field definitions
    let (generics_impl, generics) = generics_split_for_impl(parsed.generics.as_ref());
    let kind;
    let fields = match &parsed.kind {
        StructKind::Struct { clauses: _, fields } => {
            kind = "facet::StructKind::Struct";
            fields
                .content
                .0
                .iter()
                .map(|field| {
                    let field_name = field.value.name.to_string();
                    gen_struct_field(
                        &field_name,
                        &struct_name,
                        &generics,
                        &field.value.attributes,
                    )
                })
                .collect::<Vec<String>>()
        }
        StructKind::TupleStruct {
            fields,
            clauses: _,
            semi: _,
        } => {
            kind = "facet::StructKind::TupleStruct";
            fields
                .content
                .0
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let field_name = format!("{index}");
                    gen_struct_field(
                        &field_name,
                        &struct_name,
                        &generics,
                        &field.value.attributes,
                    )
                })
                .collect::<Vec<String>>()
        }
        StructKind::UnitStruct {
            clauses: _,
            semi: _,
        } => {
            kind = "facet::StructKind::Unit";
            vec![]
        }
    }
    .join(", ");

    let static_decl = if parsed.generics.is_none() {
        generate_static_decl(&struct_name)
    } else {
        String::new()
    };
    let maybe_container_doc = build_maybe_doc(&parsed.attributes);

    // Generate the impl
    let output = format!(
        r#"
{static_decl}

#[automatically_derived]
unsafe impl<{generics_impl}> facet::Facet for {struct_name}<{generics}> {{
    const SHAPE: &'static facet::Shape = &const {{
        let fields: &'static [facet::Field] = &const {{[{fields}]}};

        facet::Shape::builder()
            .id(facet::ConstTypeId::of::<Self>())
            .layout(core::alloc::Layout::new::<Self>())
            .vtable(facet::value_vtable!(
                Self,
                |f, _opts| core::fmt::Write::write_str(f, "{struct_name}")
            ))
            .def(facet::Def::Struct(facet::StructDef::builder()
                .kind({kind})
                .fields(fields)
                .build()))
            {maybe_container_doc}
            .build()
    }};
}}
        "#
    );

    output.into_token_stream().into()
}

fn generics_split_for_impl(generics: Option<&GenericParams>) -> (String, String) {
    let Some(generics) = generics else {
        return ("".to_string(), "".to_string());
    };
    let mut generics_impl = Vec::new();
    let mut generics_target = Vec::new();

    for param in generics.params.0.iter() {
        match &param.value {
            GenericParam::Type {
                name,
                bounds,
                default: _,
            } => {
                let name = name.to_string();
                let mut impl_ = name.clone();
                if let Some(bounds) = bounds {
                    impl_.push_str(&format!(": {}", VerbatimDisplay(&bounds.second)));
                }
                generics_impl.push(impl_);
                generics_target.push(name);
            }
            GenericParam::Lifetime { name, bounds } => {
                let name = name.to_string();
                let mut impl_ = name.clone();
                if let Some(bounds) = bounds {
                    impl_.push_str(&format!(": {}", VerbatimDisplay(&bounds.second)));
                }
                generics_impl.push(impl_);
                generics_target.push(name);
            }
            GenericParam::Const {
                _const,
                name,
                _colon,
                typ,
                default: _,
            } => {
                let name = name.to_string();
                generics_impl.push(format!("const {}: {}", name, VerbatimDisplay(typ)));
                generics_target.push(name);
            }
        }
    }
    let generics_impl = generics_impl.join(", ");
    let generics_target = generics_target.join(", ");
    (generics_impl, generics_target)
}
