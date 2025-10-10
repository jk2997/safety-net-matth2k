use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Derive macro for the Instantiable trait.
/// 
/// This macro works with enums where each variant wraps a type that implements Instantiable.
/// It generates an implementation that delegates all trait methods to the wrapped type.
///
/// Use the `#[instantiable(constant)]` attribute on a variant to specify which variant
/// should be used for `from_constant()`.
///
/// # Example
///
/// ```rust
/// #[derive(Debug, Clone, Instantiable)]
/// enum Cell {
///     #[instantiable(constant)]
///     Lut(Lut),
///     FlipFlop(FlipFlop),
///     Gate(Gate),
/// }
/// ```

fn impl_instantiable_trait(ast: DeriveInput) -> TokenStream {
    let ident = ast.ident;

    // Only support enums
    let variants = match ast.data {
        Data::Enum(data_enum) => data_enum.variants,
        _ => {
            return syn::Error::new_spanned(
                ident,
                "Instantiable can only be derived for enums",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract variant names and find the constant variant
    let mut variant_names = Vec::new();
    let mut constant_variant: Option<syn::Ident> = None;

    for variant in variants {
        let variant_name = &variant.ident;

        // Validate that the variant has exactly one unnamed field
        match variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {

            }
            _ => {
                return syn::Error::new_spanned(
                    variant,
                    "Each enum variant must have exactly one unnamed field",
                )
                .to_compile_error()
                .into();
            }
        }

        // Check for #[instantiable(constant)] attribute
        for attr in &variant.attrs {
            if attr.path().is_ident("instantiable") {
                if let Ok(_) = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("constant") {
                        if constant_variant.is_some() {
                            return Err(syn::Error::new_spanned(
                                attr,
                                "Only one variant can be marked with #[instantiable(constant)]"
                            ));
                        }
                        constant_variant = Some(variant_name.clone());
                        Ok(())
                    } else {
                        Err(meta.error("expected 'constant'"))
                    }
                }) {
                
                } else {
                    // Error occurred during parsing
                    return syn::Error::new_spanned(
                        attr,
                        "Failed to parse #[instantiable] attribute"
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        variant_names.push(variant_name.clone());
    }

    // Generate match arms for each method
    let get_name_arms = variant_names.iter().map(|v| {
        quote! { #ident::#v(inner) => inner.get_name() }
    });

    let get_input_ports_arms = variant_names.iter().map(|v| {
        quote! { #ident::#v(inner) => inner.get_input_ports().into_iter().collect::<Vec<_>>() }
    });

    let get_output_ports_arms = variant_names.iter().map(|v| {
        quote! { #ident::#v(inner) => inner.get_output_ports().into_iter().collect::<Vec<_>>() }
    });

    let has_parameter_arms = variant_names.iter().map(|v| {
        quote! { #ident::#v(inner) => inner.has_parameter(id) }
    });

    let get_parameter_arms = variant_names.iter().map(|v| {
        quote! { #ident::#v(inner) => inner.get_parameter(id) }
    });

    let set_parameter_arms = variant_names.iter().map(|v| {
        quote! { #ident::#v(inner) => inner.set_parameter(id, val) }
    });

    let parameters_arms = variant_names.iter().map(|v| {
        quote! { #ident::#v(inner) => inner.parameters().collect::<Vec<_>>().into_iter() }
    });

    let get_constant_arms = variant_names.iter().map(|v| {
        quote! { #ident::#v(inner) => inner.get_constant() }
    });   

    // Generate from_constant implementation based on the marked variant
    let from_constant_impl = if let Some(const_var) = constant_variant {
        quote! {
            fn from_constant(val: Logic) -> Option<Self> {
                if (val == Logic::True) || (val == Logic::False) {
                    return #const_var::from_constant(val).map(#ident::#const_var);
                } else {
                    return None;
                }
            }
        }
    } else {
        quote! {
            fn from_constant(_val: Logic) -> Option<Self> {
                None
            }
        }
    };

    // Generate the implementation
    let expanded = quote! {
        impl Instantiable for #ident {
            fn get_name(&self) -> &Identifier {
                match self {
                    #(#get_name_arms),*
                }
            }

            fn get_input_ports(&self) -> impl IntoIterator<Item = &Net> {
                match self {
                    #(#get_input_ports_arms),*
                }
            }

            fn get_output_ports(&self) -> impl IntoIterator<Item = &Net> {
                match self {
                    #(#get_output_ports_arms),*
                }
            }

            fn has_parameter(&self, id: &Identifier) -> bool {
                match self {
                    #(#has_parameter_arms),*
                }
            }

            fn get_parameter(&self, id: &Identifier) -> Option<Parameter> {
                match self {
                    #(#get_parameter_arms),*
                }
            }

            fn set_parameter(&mut self, id: &Identifier, val: Parameter) -> Option<Parameter> {
                match self {
                    #(#set_parameter_arms),*
                }
            }

            fn parameters(&self) -> impl Iterator<Item = (Identifier, Parameter)> {
                match self {
                    #(#parameters_arms),*
                }
            }

            #from_constant_impl

            fn get_constant(&self) -> Option<Logic> {
                match self {
                    #(#get_constant_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)    
            
}

#[proc_macro_derive(Instantiable, attributes(instantiable))]
pub fn inst_derive_macro(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    impl_instantiable_trait(ast)
}