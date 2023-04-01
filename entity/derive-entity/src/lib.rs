use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data::Struct, DataStruct, DeriveInput, Field, Fields::Named, FieldsNamed,
    Path, Type, TypePath,
};

struct Entity {
    name: String,
    fields: Vec<EntityField>,
}

struct EntityField {
    name: String,
    ty: String,
}

fn get_entity_field(field: &Field) -> Option<EntityField> {
    let ident = match &field.ident {
        Some(id) => format!("{}", id),
        None => {
            return None;
        }
    };

    let ty_ident = match &field.ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => segments.first().and_then(|s| Some(format!("{}", s.ident))),
        _ => {
            return None;
        }
    };
    let entity_field = EntityField {
        name: ident,
        ty: ty_ident.unwrap(),
    };
    Some(entity_field)
}

fn ident_to_entity_name(ident: syn::Ident) -> String {
    let mut s = ident.to_string();
    let first_char = s.chars().next().unwrap().to_lowercase().next().unwrap();
    s.replace_range(0..1, &first_char.to_string());
    s.push('s');
    s
}

#[proc_macro_derive(Entity)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);
    let fields = if let Struct(DataStruct {
        fields: Named(FieldsNamed { ref named, .. }),
        ..
    }) = data
    {
        named
    } else {
        panic!("This is not supported.")
    };
    let entity = Entity {
        name: ident_to_entity_name(ident.clone()),
        fields: fields.iter().filter_map(|field| get_entity_field(field)).collect(),
    };

    let entity_name = entity.name;
    let fields: Vec<String> = entity.fields.iter().map(|f| f.name.to_string()).collect();
    let tys: Vec<String> = entity.fields.iter().map(|f| f.ty.to_string()).collect();

    let result = quote! {
        impl #ident {
            pub fn entity() -> ::entity::EntityModel {
                ::entity::EntityModel { entity: #entity_name, columns: vec![#(#fields),*], tys: vec![#(#tys),*] }
            }
        }
    };
    result.into()
}
